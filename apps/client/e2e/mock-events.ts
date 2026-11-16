type Listener = (event: string, data: unknown) => void;

const listeners = new Map<string, Set<Listener>>();

/** The function that sends an event to everyone following project `id`. */
export const tell =
	(id: string): Listener =>
	(event, data) =>
		listeners.get(id)?.forEach((listener) => listener(event, data));

/** `/projects/{id}/events`: a server-sent stream that opens with `changed` at the current `rev`. */
export function events(id: string, rev: number): Response {
	let listener: Listener = () => undefined;
	const stream = new ReadableStream({
		start(controller) {
			listener = (event, data) => controller.enqueue(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
			listeners.set(id, (listeners.get(id) ?? new Set()).add(listener));
			listener('changed', { rev });
		},
		cancel: () => void listeners.get(id)?.delete(listener)
	});
	return new Response(stream, { headers: { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' } });
}
