/// Wraps a bare code snippet into a runnable `main` for languages that need one
/// (mirrors the reference Discord bot's `add_boilerplate`). No-op for languages
/// not listed here, or when the snippet already looks runnable on its own.
pub fn wrap_if_needed(language: &str, source: &str) -> String {
    match language {
        "java" => for_java(source),
        "scala" => for_scala(source),
        "rust" => for_rust(source),
        "c" | "c++" => for_c_cpp(source),
        "go" => for_go(source),
        "csharp" | "dotnet" | "c#.net" | "c#" => for_csharp(source),
        _ => source.to_string(),
    }
}

fn for_go(source: &str) -> String {
    if source.contains("main") {
        return source.to_string();
    }
    let mut imports = Vec::new();
    let mut code = vec!["func main() {".to_string()];
    for line in source.lines() {
        if line.trim_start().starts_with("import") {
            imports.push(line.to_string());
        } else {
            code.push(line.to_string());
        }
    }
    code.push("}".to_string());

    let mut out = vec!["package main".to_string()];
    out.extend(imports);
    out.extend(code);
    out.join("\n")
}

fn for_c_cpp(source: &str) -> String {
    if source.contains("main") {
        return source.to_string();
    }
    let mut imports = Vec::new();
    let mut code = vec!["int main() {".to_string()];
    for line in source.replace(';', ";\n").lines() {
        if line.trim_start().starts_with("#include") {
            imports.push(line.to_string());
        } else {
            code.push(line.to_string());
        }
    }
    code.push("}".to_string());
    imports.extend(code);
    imports.join("\n")
}

fn for_csharp(source: &str) -> String {
    if source.contains("class") {
        return source.to_string();
    }
    let has_main = source.contains("static void Main");
    let mut imports = Vec::new();
    let mut code = vec!["class Program{".to_string()];
    if !has_main {
        code.push("static void Main(string[] args){".to_string());
    }
    for line in source.replace(';', ";\n").lines() {
        if line.trim_start().starts_with("using") {
            imports.push(line.to_string());
        } else {
            code.push(line.to_string());
        }
    }
    if !has_main {
        code.push("}".to_string());
    }
    code.push("}".to_string());
    imports.extend(code);
    imports.join("\n").replace(";\n", ";")
}

fn for_java(source: &str) -> String {
    if source.contains("class") {
        return source.to_string();
    }
    let mut imports = Vec::new();
    let mut code = vec!["public class temp extends Object {public static void main(String[] args) {".to_string()];
    for line in source.replace(';', ";\n").lines() {
        if line.trim_start().starts_with("import") {
            imports.push(line.to_string());
        } else {
            code.push(line.to_string());
        }
    }
    code.push("}}".to_string());
    imports.extend(code);
    imports.join("\n").replace(";\n", ";")
}

fn for_scala(source: &str) -> String {
    const MARKERS: [&str; 4] = ["extends App", "def main", "@main def", "@main() def"];
    if MARKERS.iter().any(|m| source.contains(m)) {
        return source.to_string();
    }
    let indented = source.lines().map(|l| format!("  {l}")).collect::<Vec<_>>().join("\n");
    format!("@main def run(): Unit = {{\n{}\n}}\n", indented.trim_end())
}

fn for_rust(source: &str) -> String {
    if source.contains("fn main") {
        return source.to_string();
    }
    let mut imports = Vec::new();
    let mut code = vec!["fn main() {".to_string()];
    for line in source.replace(';', ";\n").lines() {
        if line.trim_start().starts_with("use") {
            imports.push(line.to_string());
        } else {
            code.push(line.to_string());
        }
    }
    code.push("}".to_string());
    imports.extend(code);
    imports.join("\n")
}