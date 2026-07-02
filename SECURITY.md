# Security Policy

## Supported Versions

Security fixes are provided for the latest published version on `main`.

## Reporting a Vulnerability

Open a private GitHub security advisory if available, or contact the maintainer through the GitHub profile linked from the repository.

Please include:

- affected version or commit;
- operating system;
- MCP client used;
- D2 CLI version;
- minimal input that reproduces the issue;
- expected and actual behavior.

Do not include secrets, private diagrams, tokens, or credentials in the report.

## Design Boundaries

This server does not provide a general sandbox for the D2 renderer. It narrows the MCP interface by restricting paths, timeouts, source size, output size, and remote asset usage. If you run it against untrusted input, run the MCP server itself inside an OS/container sandbox appropriate for your environment.
