Audit mode. Do not build anything new. Act as a hostile reviewer of the work done in the current phase.
1. Run `node tools/gate.mjs --phase <CURRENT_PHASE> --full` and paste the output.
2. For 10 randomly chosen DONE rows of this phase (`docs/ledger/status.csv`), open the tagged implementation and test files and answer for each: does the control/feature really work end to end? is the test able to fail? any hard-coded data, swallowed error, missing state (loading, empty, error, offline, RTL, 200% text)?
3. Search for violations the scanner cannot see: fake timing, success returned before the work, errors turned into success, secrets or plaintext in logs, UI strings or colours bypassing tokens, features reachable from the UI whose engine is incomplete.
4. Fix every real problem you find (write a failing test first), or reopen the row by asking the owner to reset it, and list what you fixed. If nothing is wrong, say exactly what you checked.
