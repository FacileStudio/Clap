import { ApiError, backend } from '$lib/backend';
import type { PublicRender } from '$lib/types';

/**
 * A shared video being watched through its link. It calls the public API only, without a session. Only a `404`
 * means the link is dead; a `429`, a network failure or any other error can be tried again.
 */
export class Watching {
	video = $state<PublicRender | null>(null);
	gone = $state(false);
	failed = $state(false);

	async load(token: string): Promise<void> {
		this.failed = false;
		try {
			this.video = await backend.publicRender(token);
			this.gone = false;
		} catch (error) {
			this.gone = error instanceof ApiError && error.status === 404;
			this.failed = !this.gone;
		}
	}
}
