/** The two things a revision asks of the page: the stored project, and showing a source that replaced its own. */
export interface RevisionIo {
	get(): Promise<{ source: string; rev: number }>;
	/** The stored source changed: the page must show `source`. */
	onreplace(source: string): void;
}

/**
 * The revision of the project the editor shows. Every write names it; a `changed` event past it loads the stored
 * project, the way an outside write by the CLI, an agent or another tab reaches the editor.
 */
export class Revision {
	rev = $state(0);
	private seen = 0;

	constructor(private readonly io: RevisionIo) {}

	/** The editor now shows the project at `rev`. */
	open(rev: number): void {
		this.rev = rev;
		this.seen = Math.max(this.seen, rev);
	}

	/** A `changed` event reported revision `rev`: a newer one is loaded. */
	async remote(rev: number): Promise<void> {
		this.seen = Math.max(this.seen, rev);
		if (this.seen > this.rev) await this.reload();
	}

	/** Loads the stored project. */
	async reload(): Promise<void> {
		const theirs = await this.io.get();
		this.open(theirs.rev);
		this.io.onreplace(theirs.source);
	}
}
