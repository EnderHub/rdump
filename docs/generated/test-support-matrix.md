# Semantic Support Matrix

Generated from the shared language-behavior tests that exercise cross-language semantic predicates.

| Suite | Languages | Fixture | Query | Language-specific |
| --- | --- | --- | --- | --- |
| js_ts_shared | javascript, typescript | mixed_project | `def:OldLogger` | no |
| js_ts_shared | javascript, typescript | mixed_project | `def:ILog | def:LogLevel` | no |
| js_ts_shared | javascript, typescript | mixed_project | `func:createLog` | no |
| js_ts_shared | javascript, typescript | mixed_project | `import:path & ext:ts` | no |
| js_ts_shared | javascript, typescript | mixed_project | `call:log & ext:js` | no |
| js_ts_shared | javascript, typescript | mixed_project | `call:log & ext:ts` | no |
| js_ts_shared | javascript, typescript | mixed_project | `comment:REVIEW` | no |
| js_ts_shared | javascript, typescript | mixed_project | `str:logging:` | no |
| js_ts_shared | javascript, typescript | mixed_project | `interface:ILog & type:LogLevel` | no |
| python_shared | python | mixed_project | `def:Helper & ext:py` | no |
| python_shared | python | mixed_project | `func:run_helper` | no |
| python_shared | python | mixed_project | `import:os & ext:py` | no |
| python_shared | python | mixed_project | `comment:FIXME & class:Helper` | no |
| python_shared | python | mixed_project | `str:/tmp/data` | no |
| python_shared | python | mixed_project | `call:run_helper | call:do_setup` | no |
| react_shared | react, jsx, tsx | mixed_project | `component:App & ext:tsx` | no |
| react_shared | react, jsx, tsx | mixed_project | `component:Button & ext:jsx` | no |
| react_shared | react, jsx, tsx | mixed_project | `element:h1 & ext:tsx` | yes |
| react_shared | react, jsx, tsx | mixed_project | `element:Button & ext:tsx` | yes |
| react_shared | react, jsx, tsx | mixed_project | `hook:useState` | yes |
| react_shared | react, jsx, tsx | mixed_project | `hook:useAuth` | yes |
| react_shared | react, jsx, tsx | mixed_project | `customhook:useAuth` | yes |
| react_shared | react, jsx, tsx | mixed_project | `prop:onClick` | yes |
| react_shared | react, jsx, tsx | mixed_project | `element:Button & prop:disabled` | yes |
