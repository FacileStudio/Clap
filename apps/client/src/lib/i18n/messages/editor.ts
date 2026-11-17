import type { Row } from '../row';

/** The editor: tabs, media bin, preview, save states, the project problem notice and Bulle. */
export const editor = {
	'editor.media': ['Media', 'Média', 'Medios', 'Medien'],
	'editor.panels': ['Editor panels', 'Panneaux de l’éditeur', 'Paneles del editor', 'Editorbereiche'],
	'editor.resize': ['Resize the panels', 'Redimensionner les panneaux', 'Redimensionar los paneles', 'Bereiche anpassen'],
	'editor.compact': [
		'The timeline is read-only on a small screen. Open Clap on a wider screen to edit it.',
		'La timeline est en lecture seule sur petit écran. Ouvrez Clap sur un écran plus large pour la modifier.',
		'La línea de tiempo es de solo lectura en pantallas pequeñas. Abra Clap en una pantalla más ancha para editarla.',
		'Auf kleinen Bildschirmen ist die Timeline schreibgeschützt. Öffnen Sie Clap auf einem breiteren Bildschirm, um sie zu bearbeiten.'
	],
	'preview.title': ['Preview', 'Aperçu', 'Vista previa', 'Vorschau'],
	'preview.play': ['Play', 'Lire', 'Reproducir', 'Abspielen'],
	'preview.pause': ['Pause', 'Pause', 'Pausa', 'Pause'],
	'preview.start': ['Go to the start', 'Aller au début', 'Ir al inicio', 'Zum Anfang'],
	'preview.end': ['Go to the end', 'Aller à la fin', 'Ir al final', 'Zum Ende'],
	'preview.scrub': ['Position', 'Position', 'Posición', 'Position'],
	'preview.fit': ['Fit', 'Ajuster', 'Ajustar', 'Einpassen'],
	'preview.actual': ['100%', '100 %', '100 %', '100 %'],
	'preview.zoom': ['Preview zoom', 'Zoom de l’aperçu', 'Zoom de la vista previa', 'Vorschau-Zoom'],
	'preview.safe': ['Safe areas', 'Zones de sécurité', 'Zonas seguras', 'Sichere Bereiche'],
	'preview.timecode': ['Timecode', 'Timecode', 'Código de tiempo', 'Timecode'],
	'preview.note': [
		'Preview. The render is the reference.',
		'Aperçu. Le rendu fait foi.',
		'Vista previa. El render es la referencia.',
		'Vorschau. Maßgeblich ist das Rendering.'
	],
	'save.clean': ['Saved', 'Enregistré', 'Guardado', 'Gespeichert'],
	'save.saving': ['Saving…', 'Enregistrement…', 'Guardando…', 'Speichern…'],
	'problem.title': [
		'This project has a problem the editor cannot fix',
		'Ce projet a un problème que l’éditeur ne peut pas corriger',
		'Este proyecto tiene un problema que el editor no puede corregir',
		'Dieses Projekt hat ein Problem, das der Editor nicht beheben kann'
	],
	'problem.body': [
		'Open it with the clap command line: clap pull, then clap check to see what to fix, then clap push. Until then the timeline shows what it can read.',
		'Ouvrez-le avec la ligne de commande clap : clap pull, puis clap check pour voir quoi corriger, puis clap push. D’ici là, la timeline montre ce qu’elle peut lire.',
		'Ábralo con la línea de comandos clap: clap pull, luego clap check para ver qué corregir y después clap push. Mientras tanto, la línea de tiempo muestra lo que puede leer.',
		'Öffnen Sie es mit der clap-Kommandozeile: clap pull, dann clap check, um zu sehen, was zu beheben ist, dann clap push. Bis dahin zeigt die Timeline, was sie lesen kann.'
	],
	'problem.detail': ['What the server says', 'Ce que dit le serveur', 'Lo que dice el servidor', 'Was der Server meldet'],
	'media.preview.close': ['Close the preview', 'Fermer l’aperçu', 'Cerrar la vista previa', 'Vorschau schließen'],
	'insert.label': ['Insert', 'Insérer', 'Insertar', 'Einfügen'],
	'bulle.ask': ['Ask {name}', 'Demander à {name}', 'Preguntar a {name}', '{name} fragen'],
	'media.drop': [
		'Drop videos, sounds or pictures here',
		'Déposez ici des vidéos, des sons ou des images',
		'Suelte aquí vídeos, sonidos o imágenes',
		'Videos, Töne oder Bilder hier ablegen'
	],
	'media.browse': ['Browse', 'Parcourir', 'Examinar', 'Durchsuchen'],
	'media.empty': [
		'No media yet. Upload a file, then drag it onto the timeline.',
		'Aucun média. Envoyez un fichier, puis glissez-le sur la timeline.',
		'Sin medios. Suba un archivo y arrástrelo a la línea de tiempo.',
		'Noch keine Medien. Laden Sie eine Datei hoch und ziehen Sie sie auf die Timeline.'
	],
	'media.failed': [
		'The file could not be uploaded',
		'Le fichier n’a pas pu être envoyé',
		'No se pudo subir el archivo',
		'Die Datei konnte nicht hochgeladen werden'
	],
	'media.probing': ['Analysing…', 'Analyse…', 'Analizando…', 'Wird analysiert…'],
	'media.unreadable': ['Unreadable', 'Illisible', 'Ilegible', 'Nicht lesbar'],
	'media.add': [
		'Add {name} at the playhead',
		'Ajouter {name} à la tête de lecture',
		'Añadir {name} en el cabezal',
		'{name} am Abspielkopf einfügen'
	],
	'media.delete': ['Delete {name}', 'Supprimer {name}', 'Eliminar {name}', '{name} löschen'],
	'media.uploads': ['Uploads', 'Envois', 'Subidas', 'Uploads']
} satisfies Record<string, Row>;
