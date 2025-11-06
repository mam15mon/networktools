export function extractErrorMessage(error: unknown): string {
	if (typeof error === "string") return error;
	if (error instanceof Error) return error.message;
	if (error && typeof error === "object" && "message" in error) {
		const message = (error as { message?: unknown }).message;
		if (typeof message === "string") {
			return message;
		}
	}
	return "发生未知错误";
}
