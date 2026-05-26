/// Uses tree-sitter-python to parse function/class definitions and produce
/// PEP 257-style docstring bodies (without the surrounding triple-quotes —
/// the LSP layer adds those).
use tree_sitter::{Node, Parser};

thread_local! {
    static PARSER: std::cell::RefCell<Parser> = std::cell::RefCell::new({
        let mut p = Parser::new();
        p.set_language(&tree_sitter_python::LANGUAGE.into())
            .expect("tree-sitter-python language init failed");
        p
    });
}

// -- Public API --

/// Walk upward from `cursor_line - 1` and return the source of the nearest
/// `def` or `class` statement, or `None` if not found.
///
/// Handles multi-line signatures:
/// ```python
/// def foo(
///     a: int,
///     b: str,
/// ) -> bool:
/// ```
/// Maximum lines to scan upward when looking for a def/class header.
const MAX_SIGNATURE_LINES: usize = 50;

pub fn find_definition_above(lines: &[&str], cursor_line: usize) -> Option<String> {
    if lines.is_empty() || cursor_line == 0 {
        return None;
    }

    let search_from = cursor_line.saturating_sub(MAX_SIGNATURE_LINES);
    let mut start = None;
    for i in (search_from..cursor_line).rev() {
        let stripped = lines[i].trim_start();
        if stripped.starts_with("def ")
            || stripped.starts_with("async def ")
            || stripped.starts_with("class ")
        {
            start = Some(i);
            break;
        }
    }

    let mut start = start?;

    // Check for decorators above the def/class line
    while start > 0 {
        let prev_line = lines[start - 1].trim_start();
        if prev_line.starts_with('@') {
            start -= 1;
        } else if prev_line.is_empty() {
            // Blank lines may appear between stacked decorators; keep walking.
            start -= 1;
        } else {
            break;
        }
    }

    let source = lines[start..cursor_line].join("\n");

    // Must end with `:` — rejects stray defs separated from the cursor by a body.
    if !source.trim_end().ends_with(':') {
        return None;
    }

    Some(source)
}

/// Parse the definition source and generate a PEP 257 docstring body.
/// Returns `None` if the source can't be parsed.
pub fn generate_docstring(
    definition_source: &str,
    all_lines: &[&str],
    cursor_line: usize,
) -> Option<String> {
    PARSER.with(|cell| -> Option<String> {
        let mut parser = cell.borrow_mut();

        let raise_types = collect_raise_types(&mut parser, all_lines, cursor_line);

        // Append a dummy body so tree-sitter sees a complete function/class node
        let full_source = format!("{}\n    pass", definition_source);
        let tree = parser.parse(&full_source, None)?;
        let root = tree.root_node();

        let node = first_def_node(root)?;

        match node.kind() {
            "function_definition" => {
                build_function_docstring(node, full_source.as_bytes(), raise_types)
            }
            "class_definition" => {
                build_class_docstring(full_source.as_bytes(), None, all_lines, cursor_line)
            }
            "decorated_definition" => {
                let inner = node.child_by_field_name("definition")?;
                match inner.kind() {
                    "function_definition" => {
                        build_function_docstring(inner, full_source.as_bytes(), raise_types)
                    }
                    "class_definition" => {
                        // Pass the parent decorated_definition node so we can check decorators
                        build_class_docstring(
                            full_source.as_bytes(),
                            Some(node),
                            all_lines,
                            cursor_line,
                        )
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    })
}

// -- Tree-sitter helpers --

fn first_def_node<'a>(node: Node<'a>) -> Option<Node<'a>> {
    if matches!(
        node.kind(),
        "function_definition" | "class_definition" | "decorated_definition"
    ) {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = first_def_node(child) {
            return Some(found);
        }
    }
    None
}

fn node_text<'a>(node: Node, src: &'a [u8]) -> &'a str {
    std::str::from_utf8(&src[node.byte_range()]).unwrap_or("")
}

/// Collect unique exception type names from raise statements in the function body.
fn collect_raise_types(
    parser: &mut tree_sitter::Parser,
    all_lines: &[&str],
    cursor_line: usize,
) -> Vec<String> {
    if cursor_line == 0 || cursor_line >= all_lines.len() {
        return vec![];
    }

    let def_line = all_lines.get(cursor_line.saturating_sub(1)).unwrap_or(&"");
    let def_indent = def_line.len() - def_line.trim_start().len();

    let mut body_lines = Vec::new();
    for &line in all_lines[cursor_line + 1..].iter() {
        let trimmed = line.trim_start();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            body_lines.push(line);
            continue;
        }

        let line_indent = line.len() - trimmed.len();

        if line_indent <= def_indent {
            break;
        }

        body_lines.push(line);
    }

    if body_lines.is_empty() {
        return vec![];
    }

    let body_text = body_lines.join("\n");
    let tree = match parser.parse(&body_text, None) {
        Some(t) => t,
        None => return vec![],
    };

    let mut types: Vec<String> = Vec::new();
    extract_raise_types(tree.root_node(), body_text.as_bytes(), &mut types, None);
    types
}

/// Recursively walk the AST collecting unique exception type names from raise statements.
/// Does not descend into nested function or class definitions.
/// `except_context` carries the caught exception type name when inside an `except` clause,
/// so that a bare `raise` (re-raise) can be attributed to the right type.
fn extract_raise_types(
    node: Node,
    src: &[u8],
    types: &mut Vec<String>,
    except_context: Option<String>,
) {
    if node.kind() == "raise_statement" {
        if let Some(exc_expr) = node.named_child(0) {
            let name = match exc_expr.kind() {
                "call" => exc_expr
                    .child_by_field_name("function")
                    .map(|f| node_text(f, src).to_string()),
                "identifier" | "attribute" => Some(node_text(exc_expr, src).to_string()),
                _ => None,
            };
            if let Some(t) = name {
                if !types.contains(&t) {
                    types.push(t);
                }
            }
        } else if let Some(exc_type) = &except_context {
            // Bare `raise` re-raises the exception from the enclosing `except` clause.
            if !types.contains(exc_type) {
                types.push(exc_type.clone());
            }
        }
        return;
    }

    // Raises inside nested functions/classes belong to those scopes, not the outer function.
    if matches!(
        node.kind(),
        "function_definition" | "class_definition" | "decorated_definition"
    ) {
        return;
    }

    // When entering an `except` clause, capture the caught type for any bare `raise` inside it.
    let new_context = if node.kind() == "except_clause" {
        node.named_child(0)
            .filter(|n| matches!(n.kind(), "identifier" | "attribute"))
            .map(|n| node_text(n, src).to_string())
            .or_else(|| except_context.clone())
    } else {
        except_context.clone()
    };

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        extract_raise_types(child, src, types, new_context.clone());
    }
}

// -- Docstring builders --

fn format_arg_line(
    name: &str,
    annotation: Option<&str>,
    default: Option<&str>,
    counter: &mut u32,
) -> String {
    let type_part = if let Some(t) = annotation {
        format!(" ({})", t)
    } else {
        let n = *counter;
        *counter += 1;
        format!(" (${{{}:type}})", n)
    };
    let default_part = default
        .map(|d| format!(", optional (default: {})", d))
        .unwrap_or_default();
    let desc = *counter;
    *counter += 1;
    format!(
        "    {}{}: ${{{}:Description{}.}}",
        name, type_part, desc, default_part
    )
}

fn append_args_section(
    lines: &mut Vec<String>,
    args: &[(String, Option<String>, Option<String>)],
    counter: &mut u32,
) {
    if args.is_empty() {
        return;
    }
    lines.push(String::new());
    lines.push("Args:".to_string());
    for (name, annotation, default) in args {
        lines.push(format_arg_line(
            name,
            annotation.as_deref(),
            default.as_deref(),
            counter,
        ));
    }
}

fn build_function_docstring(node: Node, src: &[u8], raise_types: Vec<String>) -> Option<String> {
    let params_node = node.child_by_field_name("parameters")?;
    let return_node = node.child_by_field_name("return_type");

    let args = collect_args(params_node, src);
    let return_type = return_node.map(|n| {
        node_text(n, src)
            .trim_start_matches("->")
            .trim()
            .to_string()
    });

    let has_return = return_type
        .as_deref()
        .is_some_and(|r| r != "None" && r != "none" && !r.is_empty());

    // One-liner: no args, no return, no raises — mirrors PEP 257 one-liner style.
    if args.is_empty() && !has_return && raise_types.is_empty() {
        return Some("${1:Summary.}".to_string());
    }

    let mut lines: Vec<String> = Vec::new();
    lines.push("\n${1:Summary.}".to_string());

    let mut counter: u32 = 2;
    append_args_section(&mut lines, &args, &mut counter);

    if has_return {
        lines.push(String::new());
        lines.push("Returns:".to_string());
        lines.push(format!(
            "    {}: ${{{}:Description.}}",
            return_type.as_deref().unwrap_or(""),
            counter
        ));
        counter += 1;
    }

    if !raise_types.is_empty() {
        lines.push(String::new());
        lines.push("Raises:".to_string());
        for exc_type in &raise_types {
            lines.push(format!("    {}: ${{{}:Description.}}", exc_type, counter));
            counter += 1;
        }
    }

    Some(lines.join("\n"))
}

fn build_class_docstring(
    src: &[u8],
    parent: Option<Node>,
    all_lines: &[&str],
    cursor_line: usize,
) -> Option<String> {
    let is_dataclass = parent
        .map(|p| is_dataclass_decorated(p, src))
        .unwrap_or(false);

    if is_dataclass {
        let fields = collect_dataclass_fields_from_lines(all_lines, cursor_line);
        let mut lines: Vec<String> = Vec::new();
        lines.push("\n${1:Summary.}".to_string());

        if !fields.is_empty() {
            lines.push(String::new());
            lines.push("Attributes:".to_string());
            let mut counter: u32 = 2;
            for (name, annotation, default) in fields {
                lines.push(format_field_line(
                    &name,
                    annotation.as_deref(),
                    default.as_deref(),
                    &mut counter,
                ));
            }
        }

        Some(lines.join("\n"))
    } else {
        // Per PEP 257, regular class docstrings should describe the class itself,
        // not document __init__ parameters. Those belong in __init__'s docstring.
        // No leading newline signals to the caller to emit an inline one-liner.
        Some("${1:Summary.}".to_string())
    }
}

// -- Dataclass helpers --

fn is_dataclass_decorated(decorated_node: Node, src: &[u8]) -> bool {
    let mut cursor = decorated_node.walk();
    for child in decorated_node.children(&mut cursor) {
        if child.kind() == "decorator" {
            let text = node_text(child, src);
            // Match @dataclass or @dataclasses.dataclass
            if text.contains("dataclass") {
                return true;
            }
        }
    }
    false
}

/// Finds the first `=` at bracket depth 0, skipping `=` inside annotations like `Annotated[int, Field(ge=0)]`.
fn find_toplevel_eq(s: &str) -> Option<usize> {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '[' | '(' | '{' => depth += 1,
            ']' | ')' | '}' => depth -= 1,
            '=' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// Returns (name, type_annotation, default_value) for each field in the dataclass body.
fn collect_dataclass_fields_from_lines(
    all_lines: &[&str],
    cursor_line: usize,
) -> Vec<(String, Option<String>, Option<String>)> {
    let mut fields = Vec::new();

    if cursor_line == 0 || cursor_line >= all_lines.len() {
        return fields;
    }

    // Get the class indentation level
    let class_line = all_lines.get(cursor_line.saturating_sub(1)).unwrap_or(&"");
    let class_indent = class_line.len() - class_line.trim_start().len();

    // Expected field indentation (one level deeper than class)
    let field_indent = class_indent + 4;

    // Scan lines after cursor for field definitions
    for &line in all_lines[cursor_line + 1..].iter() {
        let trimmed = line.trim_start();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let line_indent = line.len() - trimmed.len();

        // If we're back at or before the class indentation, we've left the class body
        if line_indent <= class_indent {
            break;
        }

        // Only look at lines at the field indentation level
        if line_indent != field_indent {
            continue;
        }

        // Skip methods (def/async def)
        if trimmed.starts_with("def ") || trimmed.starts_with("async def ") {
            break;
        }

        // Parse field definition: name: type = default or name: type
        if let Some(colon_pos) = trimmed.find(':') {
            let name = trimmed[..colon_pos].trim().to_string();
            let rest = &trimmed[colon_pos + 1..];

            // Check for = to separate type from default (depth-0 to skip `=` inside annotations)
            let (type_str, default_str) = if let Some(eq_pos) = find_toplevel_eq(rest) {
                (
                    rest[..eq_pos].trim(),
                    Some(rest[eq_pos + 1..].trim().to_string()),
                )
            } else {
                (rest.trim(), None)
            };

            let annotation = if !type_str.is_empty() {
                Some(type_str.to_string())
            } else {
                None
            };

            // Skip ClassVar and InitVar (they're not instance attributes)
            if let Some(ref ann) = annotation {
                if ann.starts_with("ClassVar") || ann.starts_with("InitVar") {
                    continue;
                }
            }

            fields.push((name, annotation, default_str));
        }
    }

    fields
}

/// Format a field line for Attributes section
fn format_field_line(
    name: &str,
    annotation: Option<&str>,
    default: Option<&str>,
    counter: &mut u32,
) -> String {
    let type_part = if let Some(t) = annotation {
        format!(" ({})", t)
    } else {
        let n = *counter;
        *counter += 1;
        format!(" (${{{}:type}})", n)
    };
    let default_part = default
        .map(|d| format!(", optional (default: {})", d))
        .unwrap_or_default();
    let desc = *counter;
    *counter += 1;
    format!(
        "    {}{}: ${{{}:Description{}.}}",
        name, type_part, desc, default_part
    )
}

// -- Argument collection --

/// Returns (name, type_annotation, default_value) for each parameter,
/// skipping `self` and `cls`.
fn collect_args(params_node: Node, src: &[u8]) -> Vec<(String, Option<String>, Option<String>)> {
    let mut args = Vec::new();
    let mut cursor = params_node.walk();

    for child in params_node.children(&mut cursor) {
        match child.kind() {
            "identifier" => {
                // Positional arg with no annotation
                let name = node_text(child, src).to_string();
                if name == "self" || name == "cls" {
                    continue;
                }
                args.push((name, None, None));
            }
            "typed_parameter" => {
                // The identifier is the first child, not a named field in tree-sitter-python
                let name = child
                    .child(0)
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_default();
                if name == "self" || name == "cls" {
                    continue;
                }
                let annotation = child
                    .child_by_field_name("type")
                    .map(|n| node_text(n, src).to_string());
                args.push((name, annotation, None));
            }
            "default_parameter" => {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_default();
                if name == "self" || name == "cls" {
                    continue;
                }
                let default = child
                    .child_by_field_name("value")
                    .map(|n| node_text(n, src).to_string());
                args.push((name, None, default));
            }
            "typed_default_parameter" => {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| node_text(n, src).to_string())
                    .unwrap_or_default();
                if name == "self" || name == "cls" {
                    continue;
                }
                let annotation = child
                    .child_by_field_name("type")
                    .map(|n| node_text(n, src).to_string());
                let default = child
                    .child_by_field_name("value")
                    .map(|n| node_text(n, src).to_string());
                args.push((name, annotation, default));
            }
            "list_splat_pattern" | "dictionary_splat_pattern" => {
                // *args / **kwargs
                let mut inner = child.walk();
                for grandchild in child.children(&mut inner) {
                    if grandchild.kind() == "identifier" {
                        let name = node_text(grandchild, src).to_string();
                        let prefix = if child.kind() == "dictionary_splat_pattern" {
                            "**"
                        } else {
                            "*"
                        };
                        args.push((format!("{}{}", prefix, name), None, None));
                    }
                }
            }
            _ => {}
        }
    }

    args
}

// -- Tests --

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(s: &str) -> Vec<&str> {
        s.lines().collect()
    }

    #[test]
    fn test_simple_function() {
        let src = "def greet(name: str, times: int = 1) -> str:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1; // Position of the """ line
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("name (str)"));
        assert!(doc.contains("times (int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("str"));
    }

    #[test]
    fn test_no_params_no_return_is_oneliner() {
        let src = "def outer():\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        // No args, no return, no raises → one-liner (no leading newline)
        assert!(!doc.starts_with('\n'));
        assert!(doc.contains("${1:Summary.}"));
        assert!(!doc.contains("Args:"));
        assert!(!doc.contains("Returns:"));
    }

    #[test]
    fn test_no_annotations() {
        let src = "def add(a, b):\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("a (${"));
        assert!(doc.contains("b (${"));
    }

    #[test]
    fn test_skips_self() {
        let src = "def save(self, path: str) -> None:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("self"));
        assert!(doc.contains("path (str)"));
    }

    #[test]
    fn test_none_return_omitted() {
        let src = "def reset(self) -> None:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("Returns:"));
    }

    #[test]
    fn test_class() {
        let src = "class MyModel:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("${1")); // has summary placeholder
    }

    #[test]
    fn test_multiline_signature() {
        let src = "def foo(\n    a: int,\n    b: str,\n) -> bool:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("a (int)"));
        assert!(doc.contains("b (str)"));
        assert!(doc.contains("Returns:"));
    }

    #[test]
    fn test_class_with_init_args() {
        // Per PEP 257, class docstrings should not include __init__ args
        let src = "class Point:\n    def __init__(self, x: float, y: float):\n        pass";
        let ls = lines(src);
        let doc = generate_docstring(src, &ls, 0).unwrap();
        assert!(doc.contains("${1")); // has summary placeholder
        assert!(!doc.contains("x (")); // should NOT contain __init__ args
        assert!(!doc.contains("Args:"));
    }

    #[test]
    fn test_init_method_docstring() {
        // __init__ should be documented like any other method
        let src = "def __init__(self, x: float, y: float):\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("x (float)"));
        assert!(doc.contains("y (float)"));
        assert!(!doc.contains("self")); // self should be skipped
        assert!(doc.contains("Args:"));
    }

    // -- Complex Type Annotation Tests --

    #[test]
    fn test_union_type_old_syntax() {
        let src = "def process_value(value: Union[str, int], default: Union[str, int] = 0) -> Union[str, int]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("value (Union[str, int])"));
        assert!(doc.contains("default (Union[str, int])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Union[str, int]"));
    }

    #[test]
    fn test_union_type_pipe_syntax() {
        let src =
            "def process_value(value: str | int, default: str | int = 0) -> str | int:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("value (str | int)"));
        assert!(doc.contains("default (str | int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("str | int"));
    }

    #[test]
    fn test_optional_type() {
        let src = "def find_user(user_id: int, cache: Optional[dict] = None) -> Optional[str]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("user_id (int)"));
        assert!(doc.contains("cache (Optional[dict])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Optional[str]"));
    }

    #[test]
    fn test_callable_type() {
        let src = "def register_callback(\n    callback: Callable[[int, str], bool],\n    fallback: Callable[[str], None] = None,\n) -> None:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("callback (Callable[[int, str], bool])"));
        assert!(doc.contains("fallback (Callable[[str], None])"));
        assert!(!doc.contains("Returns:")); // None return should be omitted
    }

    #[test]
    fn test_nested_generic_types() {
        let src = "def merge_data(\n    data: list[dict[str, Any]],\n    overrides: dict[str, list[int]] = None,\n) -> list[dict[str, Any]]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("data (list[dict[str, Any]])"));
        assert!(doc.contains("overrides (dict[str, list[int]])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("list[dict[str, Any]]"));
    }

    #[test]
    fn test_generic_typevar() {
        let src = "def get_first(items: List[T], default: T = None) -> T:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("items (List[T])"));
        assert!(doc.contains("default (T)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("T:"));
    }

    #[test]
    fn test_very_long_type_annotation() {
        let src = "def complex_processor(\n    data: dict[str, Union[int, str, list[dict[str, Any]]]],\n    validators: list[Callable[[dict[str, Any]], bool]],\n) -> tuple[dict[str, Any], list[str]]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        // Verify the long types are preserved as-is
        assert!(doc.contains("data (dict[str, Union[int, str, list[dict[str, Any]]]])"));
        assert!(doc.contains("validators (list[Callable[[dict[str, Any]], bool]])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("tuple[dict[str, Any], list[str]]"));
    }

    #[test]
    fn test_multiple_type_parameters() {
        let src = "def transform_dict(data: Dict[K, V], transformer: Callable[[V], V]) -> Dict[K, V]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("data (Dict[K, V])"));
        assert!(doc.contains("transformer (Callable[[V], V])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Dict[K, V]"));
    }

    #[test]
    fn test_literal_type() {
        let src = r#"def set_mode(mode: Literal["read", "write", "append"]) -> None:
    """"#;
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains(r#"mode (Literal["read", "write", "append"])"#));
        assert!(!doc.contains("Returns:")); // None return should be omitted
    }

    #[test]
    fn test_nested_callable() {
        let src = "def higher_order_function(\n    func: Callable[[Callable[[int], str]], list[str]],\n) -> Callable[[int], str]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("func (Callable[[Callable[[int], str]], list[str]])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Callable[[int], str]"));
    }

    #[test]
    fn test_union_with_many_types() {
        let src = "def multi_type_handler(\n    value: Union[str, int, float, bool, list, dict, None],\n) -> Union[str, int, float, bool, list, dict, None]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("value (Union[str, int, float, bool, list, dict, None])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Union[str, int, float, bool, list, dict, None]"));
    }

    #[test]
    fn test_complex_nested_optional_union() {
        let src = "def parse_config(\n    config: Optional[Union[str, dict[str, Union[str, int, list[str]]]]] = None,\n) -> dict[str, Any]:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(
            doc.contains("config (Optional[Union[str, dict[str, Union[str, int, list[str]]]]])")
        );
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("dict[str, Any]"));
    }

    // -- Nested Function Tests --

    #[test]
    fn test_nested_function_basic() {
        let src = "def outer():\n    def inner(x: int) -> str:\n        \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("x (int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("str"));
    }

    #[test]
    fn test_nested_function_with_outer_params() {
        let src = "def outer(a: str, b: int):\n    def inner(x: float, y: bool = True) -> list:\n        \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        // Should document the inner function, not the outer
        assert!(doc.contains("x (float)"));
        assert!(doc.contains("y (bool)"));
        assert!(!doc.contains("a (")); // outer params should not appear
        assert!(!doc.contains("b ("));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("list"));
    }

    #[test]
    fn test_deeply_nested_function() {
        let src = "def level1():\n    def level2():\n        def level3(z: dict) -> None:\n            \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("z (dict)"));
        assert!(!doc.contains("Returns:")); // None should be omitted
    }

    #[test]
    fn test_nested_function_multiline() {
        let src = "def outer():\n    def inner(\n        x: int,\n        y: str,\n    ) -> bool:\n        \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("x (int)"));
        assert!(doc.contains("y (str)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("bool"));
    }

    #[test]
    fn test_nested_async_function() {
        let src = "def outer():\n    async def inner(url: str) -> dict:\n        \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("url (str)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("dict"));
    }

    #[test]
    fn test_nested_function_complex_types() {
        let src = "def outer():\n    def inner(data: list[dict[str, Any]]) -> Optional[str]:\n        \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("data (list[dict[str, Any]])"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Optional[str]"));
    }

    // -- Raises Section Tests --

    #[test]
    fn test_function_with_raise() {
        let src = "def divide(a: int, b: int) -> float:\n    \"\"\"\n    if b == 0:\n        raise ZeroDivisionError(\"Cannot divide by zero\")";
        let ls = lines(src);
        let cursor_line = 1; // Position of the """ line
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("a (int)"));
        assert!(doc.contains("b (int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("float"));
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("ZeroDivisionError"));
    }

    #[test]
    fn test_function_without_raise() {
        let src = "def add(a: int, b: int) -> int:\n    \"\"\"\n    return a + b";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("a (int)"));
        assert!(doc.contains("b (int)"));
        assert!(doc.contains("Returns:"));
        assert!(!doc.contains("Raises:")); // Should NOT have Raises section
    }

    #[test]
    fn test_function_with_multiple_raises() {
        let src = "def process(data: str) -> dict:\n    \"\"\"\n    if not data:\n        raise ValueError(\"Empty data\")\n    if len(data) > 100:\n        raise RuntimeError(\"Too large\")";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("data (str)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("ValueError"));
        assert!(doc.contains("RuntimeError"));
    }

    #[test]
    fn test_raises_with_no_return() {
        let src = "def validate(value: int) -> None:\n    \"\"\"\n    if value < 0:\n        raise ValueError(\"Must be positive\")";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("value (int)"));
        assert!(!doc.contains("Returns:")); // None return omitted
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("ValueError"));
    }

    #[test]
    fn test_nested_function_with_raise() {
        let src = "def outer():\n    def inner(x: int) -> str:\n        \"\"\"\n        if x < 0:\n            raise ValueError(\"Negative\")";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("x (int)"));
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("ValueError"));
    }

    #[test]
    fn test_raise_in_string_not_detected() {
        // The word "raise" in a string should NOT trigger Raises section
        let src = "def message() -> str:\n    \"\"\"\n    return \"raise the flag\"";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("Raises:")); // Should NOT have Raises section
    }

    #[test]
    fn test_raise_in_comment_not_detected() {
        // The word "raise" in a comment should NOT trigger Raises section
        let src =
            "def process() -> None:\n    \"\"\"\n    # TODO: raise an exception later\n    pass";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("Raises:")); // Should NOT have Raises section
    }

    #[test]
    fn test_bare_reraise_uses_except_type() {
        let src = "def wrapper(value: int) -> int:\n    \"\"\"\n    try:\n        return int(value)\n    except ValueError:\n        raise";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("ValueError"));
    }

    #[test]
    fn test_raise_in_nested_function_not_attributed_to_outer() {
        // The outer function does not raise; the raise is inside a nested function.
        let src = "def outer(threshold: int):\n    \"\"\"\n    def inner(item: int) -> bool:\n        if item < threshold:\n            raise ValueError(\"below threshold\")\n        return True\n    return inner";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("Raises:"));
        assert!(!doc.contains("ValueError"));
    }

    #[test]
    fn test_raise_in_try_except() {
        // Actual raise statement in try/except block should be detected
        let src = "def handle() -> None:\n    \"\"\"\n    try:\n        pass\n    except Exception:\n        raise RuntimeError(\"Failed\")";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Raises:"));
        assert!(doc.contains("RuntimeError"));
    }

    // -- Dataclass Tests --

    #[test]
    fn test_basic_dataclass() {
        let src = "@dataclass\nclass Point:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("${1:Summary.}"));
        // No fields defined yet, so no Attributes section
    }

    #[test]
    fn test_dataclass_with_fields() {
        let src = "@dataclass\nclass Point:\n    \"\"\"\n    x: float\n    y: float";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("x (float)"));
        assert!(doc.contains("y (float)"));
    }

    #[test]
    fn test_dataclass_with_defaults() {
        let src = "@dataclass\nclass Person:\n    \"\"\"\n    name: str\n    age: int = 0";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("name (str)"));
        assert!(doc.contains("age (int)"));
        assert!(doc.contains("default: 0"));
    }

    #[test]
    fn test_dataclass_with_optional() {
        let src =
            "@dataclass\nclass User:\n    \"\"\"\n    name: str\n    email: Optional[str] = None";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("name (str)"));
        assert!(doc.contains("email (Optional[str])"));
        assert!(doc.contains("default: None"));
    }

    #[test]
    fn test_dataclass_skips_classvar() {
        let src =
            "@dataclass\nclass Counter:\n    \"\"\"\n    count: int\n    total: ClassVar[int] = 0";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("count (int)"));
        assert!(!doc.contains("total")); // ClassVar should be skipped
    }

    #[test]
    fn test_dataclass_skips_initvar() {
        let src =
            "@dataclass\nclass Item:\n    \"\"\"\n    value: int\n    init_param: InitVar[str]";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("value (int)"));
        assert!(!doc.contains("init_param")); // InitVar should be skipped
    }

    #[test]
    fn test_dataclass_complex_types() {
        let src = "@dataclass\nclass Config:\n    \"\"\"\n    data: list[dict[str, Any]]\n    callback: Optional[Callable[[int], str]] = None";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("data (list[dict[str, Any]])"));
        assert!(doc.contains("callback (Optional[Callable[[int], str]])"));
    }

    #[test]
    fn test_regular_class_not_dataclass() {
        // Regular class without @dataclass should not get Attributes section
        let src = "class Point:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("Attributes:")); // Should NOT have Attributes
        assert!(doc.contains("${1:Summary.}")); // Just summary
    }

    #[test]
    fn test_dataclasses_module_decorator() {
        // Test @dataclasses.dataclass form
        let src = "@dataclasses.dataclass\nclass Point:\n    \"\"\"\n    x: float\n    y: float";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("x (float)"));
    }

    #[test]
    fn test_dataclass_annotated_field_with_default() {
        // Annotated[..., Field(ge=0)] contains `=` inside brackets — must not be
        // mistaken for the type/default separator.
        let src =
            "@dataclass\nclass Validated:\n    \"\"\"\n    score: Annotated[int, Field(ge=0)] = 0";
        let ls = lines(src);
        let cursor_line = 2;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("Attributes:"));
        assert!(doc.contains("score (Annotated[int, Field(ge=0)])"));
        assert!(doc.contains("default: 0"));
    }

    #[test]
    fn test_args_kwargs() {
        let src = "def process(*args, **kwargs):\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("*args"));
        assert!(doc.contains("**kwargs"));
        assert!(doc.contains("Args:"));
    }

    #[test]
    fn test_skips_cls() {
        let src = "def create(cls, name: str) -> \"MyClass\":\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(!doc.contains("cls"));
        assert!(doc.contains("name (str)"));
    }

    #[test]
    fn test_top_level_async_def() {
        let src = "async def fetch(url: str, timeout: int = 30) -> dict:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("url (str)"));
        assert!(doc.contains("timeout (int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("dict"));
    }

    #[test]
    fn test_keyword_only_params() {
        let src = "def configure(host: str, *, port: int = 8080, debug: bool = False) -> None:\n    \"\"\"";
        let ls = lines(src);
        let cursor_line = ls.len() - 1;
        let def = find_definition_above(&ls, cursor_line).unwrap();
        let doc = generate_docstring(&def, &ls, cursor_line).unwrap();
        assert!(doc.contains("host (str)"));
        assert!(doc.contains("port (int)"));
        assert!(doc.contains("debug (bool)"));
        assert!(!doc.contains("Returns:")); // None return omitted
    }
}
