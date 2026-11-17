import type { Row } from '@facile/muse';

/** The words drawn inside the faux editor of the hero and reused by the tiles, in the order en, fr, es, de. */
export const demo = {
	'demo.label': [
		'In the Clap editor, a clip is dropped on the timeline and trimmed, a title typed in the clip settings animates in the preview, then the render reaches an MP4.',
		'Dans l’éditeur Clap, un clip est déposé sur la timeline puis rogné, un titre tapé dans les réglages du clip s’anime dans l’aperçu, puis le rendu aboutit à un MP4.',
		'En el editor de Clap, se suelta un clip en la línea de tiempo y se recorta, un título escrito en los ajustes del clip se anima en la vista previa y el render termina en un MP4.',
		'Im Clap-Editor wird ein Clip auf die Timeline gelegt und gekürzt, ein in den Clip-Einstellungen getippter Titel erscheint animiert in der Vorschau, dann endet das Rendering in einem MP4.'
	],
	'demo.project': ['Summer in Brittany', 'Été en Bretagne', 'Verano en Bretaña', 'Sommer in der Bretagne'],
	'demo.saved': ['Saved', 'Enregistré', 'Guardado', 'Gespeichert'],
	'demo.render': ['Render', 'Rendre', 'Renderizar', 'Rendern'],
	'demo.media': ['Media', 'Média', 'Medios', 'Medien'],
	'demo.clip': ['Clip', 'Clip', 'Clip', 'Clip'],
	'demo.text': ['Text', 'Texte', 'Texto', 'Text'],
	'demo.start': ['Start', 'Début', 'Inicio', 'Start'],
	'demo.length': ['Length', 'Durée', 'Duración', 'Länge'],
	'demo.scene': ['Scene 1', 'Scène 1', 'Escena 1', 'Szene 1'],
	'demo.video': ['beach.mp4', 'plage.mp4', 'playa.mp4', 'strand.mp4'],
	'demo.audio': ['music.mp3', 'musique.mp3', 'musica.mp3', 'musik.mp3'],
	'demo.clip.title': ['Title', 'Titre', 'Título', 'Titel'],
	'demo.headline': ['Brittany, August', 'Bretagne, août', 'Bretaña, agosto', 'Bretagne, August'],
	'demo.render.running': ['Rendering', 'En cours', 'Renderizando', 'Wird gerendert'],
	'demo.render.done': ['Done', 'Terminé', 'Terminado', 'Fertig'],
	'demo.render.progress': [
		'Frame {frame} of {frames}',
		'Image {frame} sur {frames}',
		'Fotograma {frame} de {frames}',
		'Bild {frame} von {frames}'
	],
	'demo.render.file': ['summer-in-brittany.mp4', 'ete-en-bretagne.mp4', 'verano-en-bretana.mp4', 'sommer-in-der-bretagne.mp4'],
	'demo.render.download': ['Download the MP4', 'Télécharger le MP4', 'Descargar el MP4', 'MP4 herunterladen']
} satisfies Record<string, Row>;
