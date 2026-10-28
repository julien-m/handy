### Plan: Smart paste technical plan generated

- **Type:** Feature
- **Spec modified:** Yes (lifecycle status only: Draft to Planned via finalize; normative requirements unchanged)
- **Code modified:** None (plan-only run)
- **AC impacted:** None (pre-implementation)
- **Author:** Codex gpt-6.1-sol
- **Plan:** 9 implementation steps; 2 sequence diagrams, 1 state diagram, 1 logical ER diagram.
- **Decisions:** asynchronous AppKit/key worker; bounded read-only snapshot helper; guarded restoration and copy mode; 008 output-domain reuse and one-shot migration; six truthful outcomes.
- **Evidence boundary:** UI/Penflow deferred by user until shared pre-spec-ship pass; native/runtime proof required later.
