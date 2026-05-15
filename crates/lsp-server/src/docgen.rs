/// docgen.rs
///
/// Uses tree-sitter-python to parse function/class definitions and produce
/// PEP 257-style docstring bodies (without the surrounding triple-quotes —
/// the LSP layer adds those).
use tree_sitter::{Node, Parser};

// ── Public API ────────────────────────────────────────────────────────────────

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
pub fn find_definition_above(lines: &[&str], cursor_line: usize) -> Option<String> {
    if cursor_line == 0 {
        return None;
    }

    // Walk upward to find the start of a def/class
    let mut start = None;
    for i in (0..cursor_line).rev() {
        let stripped = lines[i].trim_start();
        if stripped.starts_with("def ")
            || stripped.starts_with("class ")
            || stripped.starts_with("async def ")
        {
            start = Some(i);
            break;
        }
        // Stop scanning if we hit something clearly unrelated
        if i < cursor_line.saturating_sub(1)
            && !stripped.is_empty()
            && !stripped.starts_with(')')
            && !stripped.starts_with('#')
        {
            break;
        }
    }

    let start = start?;

    // Collect lines from start up to (but not including) cursor_line
    let source = lines[start..cursor_line].join("\n");
    let trimmed = source.trim_end();

    // Must end with `:` to be a valid header
    if !trimmed.ends_with(':') {
        return None;
    }

    Some(source)
}

/// Parse the definition source and generate a PEP 257 docstring body.
/// Returns `None` if the source can't be parsed.
pub fn generate_docstring(definition_source: &str) -> Option<String> {
    let mut parser = Parser::new();
    parser
        .set_language(&tree_sitter_python::LANGUAGE.into())
        .ok()?;

    // Append a dummy body so tree-sitter sees a complete function/class node
    let full_source = format!("{}\n    pass", definition_source);
    let tree = parser.parse(&full_source, None)?;
    let root = tree.root_node();

    // Find the first function_definition, async function_definition, or class_definition
    let node = first_def_node(root, full_source.as_bytes())?;

    match node.kind() {
        "function_definition" | "decorated_definition" => {
            // decorated_definition wraps async def too; drill in
            let fn_node = if node.kind() == "decorated_definition" {
                node.child_by_field_name("definition")?
            } else {
                node
            };
            build_function_docstring(fn_node, full_source.as_bytes())
        }
        "class_definition" => build_class_docstring(node, full_source.as_bytes()),
        _ => None,
    }
}

// ── Tree-sitter helpers ───────────────────────────────────────────────────────

fn first_def_node<'a>(node: Node<'a>, src: &[u8]) -> Option<Node<'a>> {
    if matches!(
        node.kind(),
        "function_definition" | "class_definition" | "decorated_definition"
    ) {
        return Some(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(found) = first_def_node(child, src) {
            return Some(found);
        }
    }
    None
}

fn node_text<'a>(node: Node, src: &'a [u8]) -> &'a str {
    std::str::from_utf8(&src[node.byte_range()]).unwrap_or("")
}

// ── Docstring builders ────────────────────────────────────────────────────────

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
    format!("    {}{}: ${{{}:Description{}.}}", name, type_part, desc, default_part)
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
        lines.push(format_arg_line(name, annotation.as_deref(), default.as_deref(), counter));
    }
}

fn build_function_docstring(node: Node, src: &[u8]) -> Option<String> {
    let params_node = node.child_by_field_name("parameters")?;
    let return_node = node.child_by_field_name("return_type");

    let args = collect_args(params_node, src);
    let return_type = return_node.map(|n| {
        node_text(n, src)
            .trim_start_matches("->")
            .trim()
            .to_string()
    });

    let mut lines: Vec<String> = Vec::new();
    lines.push("\n${1:Summary.}".to_string());

    let mut counter: u32 = 2;
    append_args_section(&mut lines, &args, &mut counter);

    if let Some(ret) = &return_type {
        if ret != "None" && ret != "none" && !ret.is_empty() {
            lines.push(String::new());
            lines.push("Returns:".to_string());
            lines.push(format!("    {}: ${{{}:Description.}}", ret, counter));
        }
    }

    Some(lines.join("\n"))
}

fn build_class_docstring(node: Node, src: &[u8]) -> Option<String> {
    let mut lines: Vec<String> = Vec::new();
    lines.push("\n${1:Summary.}".to_string());

    let body = node.child_by_field_name("body")?;
    let mut cursor = body.walk();
    let mut counter: u32 = 2;
    for child in body.children(&mut cursor) {
        if child.kind() == "function_definition" {
            if let Some(name_node) = child.child_by_field_name("name") {
                if node_text(name_node, src) == "__init__" {
                    if let Some(params) = child.child_by_field_name("parameters") {
                        append_args_section(&mut lines, &collect_args(params, src), &mut counter);
                    }
                    break;
                }
            }
        }
    }

    Some(lines.join("\n"))
}

// ── Argument collection ───────────────────────────────────────────────────────

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

// ── Tests ─────────────────────────────────────────────────────────────────────

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
        let def = find_definition_above(&ls, ls.len() - 1).unwrap();
        let doc = generate_docstring(&def).unwrap();
        assert!(doc.contains("name (str)"));
        assert!(doc.contains("times (int)"));
        assert!(doc.contains("Returns:"));
        assert!(doc.contains("str"));
    }

    #[test]
    fn test_no_annotations() {
        let src = "def add(a, b):\n    \"\"\"";
        let ls = lines(src);
        let def = find_definition_above(&ls, ls.len() - 1).unwrap();
        let doc = generate_docstring(&def).unwrap();
        assert!(doc.contains("a (${"));
        assert!(doc.contains("b (${"));
    }

    #[test]
    fn test_skips_self() {
        let src = "def save(self, path: str) -> None:\n    \"\"\"";
        let ls = lines(src);
        let def = find_definition_above(&ls, ls.len() - 1).unwrap();
        let doc = generate_docstring(&def).unwrap();
        assert!(!doc.contains("self"));
        assert!(doc.contains("path (str)"));
    }

    #[test]
    fn test_none_return_omitted() {
        let src = "def reset(self) -> None:\n    \"\"\"";
        let ls = lines(src);
        let def = find_definition_above(&ls, ls.len() - 1).unwrap();
        let doc = generate_docstring(&def).unwrap();
        assert!(!doc.contains("Returns:"));
    }

    #[test]
    fn test_class() {
        let src = "class MyModel:\n    \"\"\"";
        let ls = lines(src);
        let def = find_definition_above(&ls, ls.len() - 1).unwrap();
        let doc = generate_docstring(&def).unwrap();
        assert!(doc.contains("${1")); // has summary placeholder
    }
}
