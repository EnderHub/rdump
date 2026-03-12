#[derive(Debug, Clone, Copy)]
pub struct SupportMatrixCase {
    pub suite: &'static str,
    pub fixture: &'static str,
    pub language_scope: &'static str,
    pub query: &'static str,
    pub expected: &'static [&'static str],
    pub absent: &'static [&'static str],
    pub intentionally_language_specific: bool,
}

const JS_TS_SHARED_CASES: &[SupportMatrixCase] = &[
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "def:OldLogger",
        expected: &["logger.js", "export class OldLogger"],
        absent: &["log_utils.ts"],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "def:ILog | def:LogLevel",
        expected: &[
            "log_utils.ts",
            "interface ILog",
            r#"type LogLevel = "info" | "warn" | "error";"#,
        ],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "func:createLog",
        expected: &["log_utils.ts", "export function createLog"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "import:path & ext:ts",
        expected: &["log_utils.ts", "import * as path from 'path';"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "call:log & ext:js",
        expected: &["logger.js", "logger.log(\"init\");"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "call:log & ext:ts",
        expected: &["log_utils.ts", "console.log(newLog);"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "comment:REVIEW",
        expected: &["log_utils.ts"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "str:logging:",
        expected: &["logger.js"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "js_ts_shared",
        fixture: "mixed_project",
        language_scope: "javascript, typescript",
        query: "interface:ILog & type:LogLevel",
        expected: &["log_utils.ts"],
        absent: &[],
        intentionally_language_specific: false,
    },
];

const PYTHON_SHARED_CASES: &[SupportMatrixCase] = &[
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "def:Helper & ext:py",
        expected: &["helper.py", "class Helper"],
        absent: &["src/main.rs"],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "func:run_helper",
        expected: &["helper.py", "def run_helper()"],
        absent: &["src/main.rs"],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "import:os & ext:py",
        expected: &["helper.py", "import os"],
        absent: &["src/lib.rs"],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "comment:FIXME & class:Helper",
        expected: &["helper.py"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "str:/tmp/data",
        expected: &["helper.py"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "python_shared",
        fixture: "mixed_project",
        language_scope: "python",
        query: "call:run_helper | call:do_setup",
        expected: &["self.do_setup()", "run_helper()"],
        absent: &[],
        intentionally_language_specific: false,
    },
];

const REACT_SHARED_CASES: &[SupportMatrixCase] = &[
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "component:App & ext:tsx",
        expected: &["function App()"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "component:Button & ext:jsx",
        expected: &["export const Button"],
        absent: &[],
        intentionally_language_specific: false,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "element:h1 & ext:tsx",
        expected: &["<h1>Welcome, {user?.name}</h1>"],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "element:Button & ext:tsx",
        expected: &["<Button onClick="],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "hook:useState",
        expected: &["const [count, setCount] = useState(0);"],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "hook:useAuth",
        expected: &["const { user } = useAuth();"],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "customhook:useAuth",
        expected: &["export default function useAuth()"],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "prop:onClick",
        expected: &["<Button onClick={", "<button onClick={onClick}"],
        absent: &[],
        intentionally_language_specific: true,
    },
    SupportMatrixCase {
        suite: "react_shared",
        fixture: "mixed_project",
        language_scope: "react, jsx, tsx",
        query: "element:Button & prop:disabled",
        expected: &["App.tsx"],
        absent: &["Button.jsx"],
        intentionally_language_specific: true,
    },
];

pub fn js_ts_shared_cases() -> &'static [SupportMatrixCase] {
    JS_TS_SHARED_CASES
}

pub fn python_shared_cases() -> &'static [SupportMatrixCase] {
    PYTHON_SHARED_CASES
}

pub fn react_shared_cases() -> &'static [SupportMatrixCase] {
    REACT_SHARED_CASES
}

pub fn all_support_matrix_cases() -> Vec<&'static SupportMatrixCase> {
    JS_TS_SHARED_CASES
        .iter()
        .chain(PYTHON_SHARED_CASES.iter())
        .chain(REACT_SHARED_CASES.iter())
        .collect()
}

pub fn render_support_matrix_markdown() -> String {
    let mut out = String::from(
        "# Semantic Support Matrix\n\nGenerated from the shared language-behavior tests that exercise cross-language semantic predicates.\n\n| Suite | Languages | Fixture | Query | Language-specific |\n| --- | --- | --- | --- | --- |\n",
    );

    for case in all_support_matrix_cases() {
        out.push_str(&format!(
            "| {} | {} | {} | `{}` | {} |\n",
            case.suite,
            case.language_scope,
            case.fixture,
            case.query,
            if case.intentionally_language_specific {
                "yes"
            } else {
                "no"
            }
        ));
    }

    out
}
