/** What the scrolling timeline viewport reports. */
export interface ViewportEvents {
	/** Ctrl or Cmd with the wheel: zoom by `factor`, keeping the time `x` pixels from the viewport's left still. */
	onzoom(factor: number, x: number): void;
	/** The viewport scrolled or changed size. */
	onview(left: number, width: number): void;
}

/** Follows the wheel, the scroll and the size of `node`. A plain wheel is left to the browser, so it scrolls. */
export function watchViewport(node: HTMLElement, events: ViewportEvents): { destroy: () => void } {
	const onwheel = (event: WheelEvent) => {
		if (!event.ctrlKey && !event.metaKey) return;
		event.preventDefault();
		events.onzoom(Math.exp(-event.deltaY * 0.002), event.clientX - node.getBoundingClientRect().left);
	};
	const measure = () => events.onview(node.scrollLeft, node.clientWidth);
	const sized = new ResizeObserver(measure);
	sized.observe(node);
	node.addEventListener('wheel', onwheel, { passive: false });
	node.addEventListener('scroll', measure, { passive: true });
	return {
		destroy() {
			sized.disconnect();
			node.removeEventListener('wheel', onwheel);
			node.removeEventListener('scroll', measure);
		}
	};
}
