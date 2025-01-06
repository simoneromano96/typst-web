import { swagger } from "@elysiajs/swagger";
import { readableStreamToBytes, spawn, write } from "bun";
import { Elysia, t } from "elysia";

new Elysia()
	.use(swagger())
	.group("/api", (app) =>
		app.group("/typst", (app) =>
			app.post(
				"/compile",
				async ({ body, error, set }) => {
					const { template, jobs, variables } = body;

					const cmd = ["typst", "compile", "-", "-"];
					// cmd.push("--diagnostic-format", "short");

					if (jobs) {
						cmd.push("--jobs", jobs.toString());
					}
					if (variables) {
						for (const [key, value] of Object.entries(variables)) {
							cmd.push("--input", `${key}=${value}`);
						}
					}

					const child = spawn({
						cmd,
						stdin: "pipe",
						stdout: "pipe",
						stderr: "pipe",
					});
					child.stdin.write(template);
					child.stdin.end();

					const statusCode = await child.exited;
					const stdout = await readableStreamToBytes(child.stdout);
					const stderr = await readableStreamToBytes(child.stderr);

					if (statusCode === 0) {
                        set.headers["content-type"] = "application/pdf";
						return stdout;
					}
					const errorMessage = new TextDecoder().decode(stderr);
					return error(500, errorMessage);
				},
				{
					body: t.Object({
						template: t.String({
							description: "Template to compile",
							example: "Hello, people!",
						}),
						variables: t.Optional( t.Record(t.String(), t.String(), {
							description: "Variables to replace in the template",
						})),
						jobs: t.Optional(
							t.Integer({
								minimum: 1,
								description: "Number of jobs to run concurrently",
							}),
						),
					}),
					response: {
						200: t.Uint8Array({
							type: "application/pdf",
							description: "Compiled PDF",
							default: "File",
							extension: "application/pdf",
							format: "binary",
						}),
						500: t.String(),
					},
				},
			),
		),
	)
	.listen(3030);

console.log("Server listening on port 3030");

