import type { Row } from '../row';

/** The editor: tabs, source, media bin, preview, save states and Bulle. */
export const editor = {
	'editor.source': ['Source', 'Source', 'Fuente', 'Quelltext'],
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
	'save.dirty': ['Unsaved changes', 'Modifications non enregistrées', 'Cambios sin guardar', 'Ungespeicherte Änderungen'],
	'save.saving': ['Saving…', 'Enregistrement…', 'Guardando…', 'Speichern…'],
	'save.invalid': [
		'Not saved: fix the source',
		'Non enregistré : corrigez la source',
		'No guardado: corrija la fuente',
		'Nicht gespeichert: Quelltext korrigieren'
	],
	'save.conflict': ['Conflict', 'Conflit', 'Conflicto', 'Konflikt'],
	'save.failed': ['Save failed', 'Échec de l’enregistrement', 'Error al guardar', 'Speichern fehlgeschlagen'],
	'conflict.title': [
		'This project changed elsewhere',
		'Ce projet a changé ailleurs',
		'Este proyecto cambió en otro lugar',
		'Dieses Projekt wurde anderswo geändert'
	],
	'conflict.body': [
		'Someone else, an agent or another tab saved a newer version while you had unsaved text.',
		'Une autre personne, un agent ou un autre onglet a enregistré une version plus récente alors que votre texte n’était pas enregistré.',
		'Otra persona, un agente u otra pestaña guardó una versión más reciente mientras usted tenía texto sin guardar.',
		'Eine andere Person, ein Agent oder ein anderer Tab hat eine neuere Version gespeichert, während Ihr Text ungespeichert war.'
	],
	'conflict.reload': ['Reload theirs', 'Recharger leur version', 'Cargar su versión', 'Deren Version laden'],
	'conflict.overwrite': [
		'Keep mine and overwrite',
		'Garder la mienne et écraser',
		'Conservar la mía y sobrescribir',
		'Meine behalten und überschreiben'
	],
	'errors.title': ['Problems in the source', 'Problèmes dans la source', 'Problemas en la fuente', 'Probleme im Quelltext'],
	'errors.line': ['Line {line}:', 'Ligne {line} :', 'Línea {line}:', 'Zeile {line}:'],
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
