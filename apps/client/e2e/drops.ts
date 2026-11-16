import type { Locator } from 'playwright-core';

/** Plays a drag of `file` from the desktop over `target` at (`x`, `y`) inside it; drops it unless `hover` only. */
export async function dragFile(
	target: Locator,
	file: { name: string; type: string; bytes: Uint8Array },
	at: { x: number; y: number; hover?: boolean }
): Promise<void> {
	const box = await target.boundingBox();
	if (!box) throw new Error('nothing to drop on');
	const point = { clientX: box.x + at.x, clientY: box.y + at.y };
	await target.evaluate(
		(node, { file, point, hover }) => {
			const data = new DataTransfer();
			data.items.add(new File([new Uint8Array(file.bytes)], file.name, { type: file.type }));
			const init = { ...point, bubbles: true, cancelable: true, dataTransfer: data };
			const send = (type: string) => node.dispatchEvent(new DragEvent(type, init));
			for (const type of hover ? ['dragenter', 'dragover'] : ['dragenter', 'dragover', 'drop']) send(type);
		},
		{ file: { ...file, bytes: [...file.bytes] }, point, hover: at.hover ?? false }
	);
}
