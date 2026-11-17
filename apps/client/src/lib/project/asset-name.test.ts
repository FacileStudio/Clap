import { describe, expect, it } from 'vitest';
import { fileName, renamedAsset } from './asset-name';

describe('asset names', () => {
	it('keeps the folder unless the typed name gives a path', () => {
		expect(renamedAsset('assets/beach.mp4', ' sea.mp4 ')).toBe('assets/sea.mp4');
		expect(renamedAsset('logo.png', 'mark.png')).toBe('mark.png');
		expect(renamedAsset('assets/beach.mp4', 'media/sea.mp4')).toBe('media/sea.mp4');
		expect(renamedAsset('assets/beach.mp4', '  ')).toBe('');
		expect(fileName('assets/a/b.mp4')).toBe('b.mp4');
	});
});
