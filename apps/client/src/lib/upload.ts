import { ApiError, toApiError } from './http';
import type { Asset } from './types';

/** A running upload: the stored asset once done, and the way to stop it. */
export interface Upload {
	done: Promise<Asset>;
	abort(): void;
}

function answered(xhr: XMLHttpRequest): Response {
	return new Response(xhr.status === 204 ? null : xhr.responseText, { status: xhr.status, statusText: xhr.statusText });
}

/**
 * Sends `file` as multipart to `url` with cookies and the CSRF header, reporting the sent fraction (0 to 1).
 * `fetch` cannot report upload progress, so this one call goes through `XMLHttpRequest`.
 */
export function uploadAsset(url: string, file: File, onprogress: (fraction: number) => void): Upload {
	const xhr = new XMLHttpRequest();
	const form = new FormData();
	form.append('file', file);
	const done = new Promise<Asset>((resolve, reject) => {
		xhr.upload.onprogress = (event) => onprogress(event.lengthComputable ? event.loaded / event.total : 0);
		xhr.onerror = () => reject(new ApiError(0, 'network', 'Network error'));
		xhr.onabort = () => reject(new ApiError(0, 'network', 'Upload cancelled'));
		xhr.onload = () => {
			if (xhr.status >= 200 && xhr.status < 300) resolve(JSON.parse(xhr.responseText) as Asset);
			else void toApiError(answered(xhr)).then(reject);
		};
	});
	xhr.open('POST', url);
	xhr.withCredentials = true;
	xhr.setRequestHeader('X-Facile-CSRF', '1');
	xhr.send(form);
	return { done, abort: () => xhr.abort() };
}
