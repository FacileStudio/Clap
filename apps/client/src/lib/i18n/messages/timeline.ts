import type { Row } from '../row';

/** The timeline: its toolbar, its clips, its refusals, and the motion starters. */
export const timeline = {
	'timeline.title': ['Timeline', 'Timeline', 'Línea de tiempo', 'Timeline'],
	'timeline.tools': ['Timeline tools', 'Outils de la timeline', 'Herramientas de la línea de tiempo', 'Timeline-Werkzeuge'],
	'timeline.split': ['Split at the playhead', 'Couper à la tête de lecture', 'Dividir en el cabezal', 'Am Abspielkopf teilen'],
	'timeline.delete': ['Delete the selection', 'Supprimer la sélection', 'Eliminar la selección', 'Auswahl löschen'],
	'timeline.text': ['Add a text', 'Ajouter un texte', 'Añadir un texto', 'Text hinzufügen'],
	'timeline.text.default': ['Your text', 'Votre texte', 'Su texto', 'Ihr Text'],
	'timeline.undo': ['Undo', 'Annuler', 'Deshacer', 'Rückgängig'],
	'timeline.redo': ['Redo', 'Rétablir', 'Rehacer', 'Wiederholen'],
	'timeline.snap': ['Snapping', 'Magnétisme', 'Magnetismo', 'Einrasten'],
	'timeline.zoom.in': ['Zoom in', 'Zoom avant', 'Acercar', 'Vergrößern'],
	'timeline.zoom.out': ['Zoom out', 'Zoom arrière', 'Alejar', 'Verkleinern'],
	'timeline.zoom.fit': ['Show the whole project', 'Afficher tout le projet', 'Mostrar todo el proyecto', 'Ganzes Projekt zeigen'],
	'timeline.ruler': ['Time ruler', 'Règle de temps', 'Regla de tiempo', 'Zeitlineal'],
	'timeline.track': ['Track {track}', 'Piste {track}', 'Pista {track}', 'Spur {track}'],
	'timeline.scene': ['Scene {scene}', 'Scène {scene}', 'Escena {scene}', 'Szene {scene}'],
	'timeline.scene.move': [
		'Scene {scene}, {duration}. Drag to reorder.',
		'Scène {scene}, {duration}. Glissez pour réordonner.',
		'Escena {scene}, {duration}. Arrastre para reordenar.',
		'Szene {scene}, {duration}. Zum Umsortieren ziehen.'
	],
	'timeline.scene.resize': [
		'Length of scene {scene}',
		'Durée de la scène {scene}',
		'Duración de la escena {scene}',
		'Länge der Szene {scene}'
	],
	'timeline.scene.delete': ['Delete scene {scene}', 'Supprimer la scène {scene}', 'Eliminar la escena {scene}', 'Szene {scene} löschen'],
	'timeline.clip': [
		'{kind} {label}, track {track}, from {from} to {to}',
		'{kind} {label}, piste {track}, de {from} à {to}',
		'{kind} {label}, pista {track}, de {from} a {to}',
		'{kind} {label}, Spur {track}, von {from} bis {to}'
	],
	'timeline.empty': [
		'Drag a media file here, or add a text.',
		'Glissez un média ici, ou ajoutez un texte.',
		'Arrastre un medio aquí, o añada un texto.',
		'Ziehen Sie eine Mediendatei hierher oder fügen Sie einen Text hinzu.'
	],
	'timeline.failed': [
		'The edit was not applied',
		'La modification n’a pas été appliquée',
		'No se aplicó el cambio',
		'Die Änderung wurde nicht übernommen'
	],
	'timeline.refused.overlap': [
		'Two clips cannot share a track at the same moment',
		'Deux clips ne peuvent pas occuper la même piste au même moment',
		'Dos clips no pueden ocupar la misma pista al mismo tiempo',
		'Zwei Clips können nicht gleichzeitig auf derselben Spur liegen'
	],
	'timeline.refused.bounds': [
		'The clip would leave its scene',
		'Le clip sortirait de sa scène',
		'El clip saldría de su escena',
		'Der Clip würde seine Szene verlassen'
	],
	'timeline.stale': [
		'The project changed elsewhere. It was reloaded.',
		'Le projet a changé ailleurs. Il a été rechargé.',
		'El proyecto cambió en otro lugar. Se ha recargado.',
		'Das Projekt wurde anderswo geändert. Es wurde neu geladen.'
	],
	'timeline.blocked': [
		'Fix or save the source before editing the timeline',
		'Corrigez ou enregistrez la source avant de modifier la timeline',
		'Corrija o guarde la fuente antes de editar la línea de tiempo',
		'Korrigieren oder speichern Sie den Quelltext, bevor Sie die Timeline bearbeiten'
	],
	'kind.video': ['Video', 'Vidéo', 'Vídeo', 'Video'],
	'kind.audio': ['Audio', 'Audio', 'Audio', 'Audio'],
	'kind.image': ['Picture', 'Image', 'Imagen', 'Bild'],
	'kind.text': ['Text', 'Texte', 'Texto', 'Text'],
	'kind.html': ['Element', 'Élément', 'Elemento', 'Element'],
	'starter.title': ['Title', 'Titre', 'Título', 'Titel'],
	'starter.lower': ['Lower third', 'Bandeau de nom', 'Tercio inferior', 'Bauchbinde'],
	'starter.caption': ['Captions', 'Sous-titres', 'Subtítulos', 'Untertitel'],
	'starter.chart': ['Chart race', 'Course de barres', 'Carrera de barras', 'Balkenrennen'],
	'starter.logo': ['Logo reveal', 'Révélation de logo', 'Revelación de logo', 'Logo-Enthüllung']
} satisfies Record<string, Row>;
