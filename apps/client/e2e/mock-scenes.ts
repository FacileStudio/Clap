import { blocks, type Block } from '../src/lib/timeline/scenes';

/** One scene as the mock rewrites it: its own frontmatter keys and its body lines. */
export type Part = { meta: Record<string, string>; body: string[] };

const PROJECT_KEYS = ['title', 'theme', 'scheme', 'size', 'step_duration', 'lang'];

function parts(source: string): { project: Record<string, string>; scenes: Part[] } {
	const lines = source.split('\n');
	const found = blocks(lines).map((block: Block) => ({ meta: { ...block.meta }, body: lines.slice(block.body, block.to) }));
	const project: Record<string, string> = {};
	for (const key of PROJECT_KEYS) {
		if (found[0].meta[key] !== undefined) project[key] = found[0].meta[key];
		delete found[0].meta[key];
	}
	return { project, scenes: found };
}

/** `source` with its scenes changed by `change` and written back, the project keys kept in the headmatter. */
export function restage(source: string, change: (scenes: Part[]) => void): string {
	const { project, scenes } = parts(source);
	change(scenes);
	const keys = (meta: Record<string, string>) => Object.entries(meta).map(([key, value]) => `${key}: ${value}`);
	const head = (index: number) => (index === 0 ? keys(project) : []);
	return scenes.flatMap((scene, index) => ['---', ...head(index), ...keys(scene.meta), '---', ...scene.body]).join('\n');
}
