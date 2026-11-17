import type { Row } from '../row';

/** The right-click menus of the timeline, the stage and the media bin, and the shortcut help. */
export const menu = {
	'menu.clip': ['Clip actions', 'Actions du clip', 'Acciones del clip', 'Clip-Aktionen'],
	'menu.timeline': ['Timeline actions', 'Actions de la timeline', 'Acciones de la línea de tiempo', 'Timeline-Aktionen'],
	'menu.stage': ['Stage actions', 'Actions de la scène', 'Acciones del escenario', 'Bühnen-Aktionen'],
	'menu.media': ['Media actions', 'Actions du média', 'Acciones del medio', 'Medien-Aktionen'],
	'menu.split': ['Split at the playhead', 'Couper à la tête de lecture', 'Dividir en el cabezal', 'Am Abspielkopf teilen'],
	'menu.trim.start': [
		'Trim the start to the playhead',
		'Rogner le début à la tête de lecture',
		'Recortar el inicio hasta el cabezal',
		'Anfang bis zum Abspielkopf kürzen'
	],
	'menu.trim.end': [
		'Trim the end to the playhead',
		'Rogner la fin à la tête de lecture',
		'Recortar el final hasta el cabezal',
		'Ende bis zum Abspielkopf kürzen'
	],
	'menu.cut': ['Cut', 'Couper', 'Cortar', 'Ausschneiden'],
	'menu.copy': ['Copy', 'Copier', 'Copiar', 'Kopieren'],
	'menu.paste': ['Paste at the playhead', 'Coller à la tête de lecture', 'Pegar en el cabezal', 'Am Abspielkopf einfügen'],
	'menu.paste.here': ['Paste here', 'Coller ici', 'Pegar aquí', 'Hier einfügen'],
	'menu.duplicate': ['Duplicate', 'Dupliquer', 'Duplicar', 'Duplizieren'],
	'menu.track.up': ['Move to the track above', 'Monter d’une piste', 'Subir una pista', 'Eine Spur nach oben'],
	'menu.track.down': ['Move to the track below', 'Descendre d’une piste', 'Bajar una pista', 'Eine Spur nach unten'],
	'menu.fade': ['Fade in and out', 'Fondu en entrée et en sortie', 'Fundido de entrada y salida', 'Ein- und ausblenden'],
	'menu.fade.clear': ['Remove the transitions', 'Retirer les transitions', 'Quitar las transiciones', 'Übergänge entfernen'],
	'menu.reveal': ['Show in the media', 'Afficher dans les médias', 'Mostrar en los medios', 'In den Medien zeigen'],
	'menu.delete': ['Delete', 'Supprimer', 'Eliminar', 'Löschen'],
	'menu.ripple': [
		'Delete and close the gap',
		'Supprimer et refermer le vide',
		'Eliminar y cerrar el hueco',
		'Löschen und Lücke schließen'
	],
	'menu.text.here': ['Add a text here', 'Ajouter un texte ici', 'Añadir un texto aquí', 'Hier Text hinzufügen'],
	'menu.media.here': ['Add media here…', 'Ajouter un média ici…', 'Añadir un medio aquí…', 'Hier Medien hinzufügen…'],
	'menu.playhead.here': [
		'Move the playhead here',
		'Placer la tête de lecture ici',
		'Mover el cabezal aquí',
		'Abspielkopf hierher setzen'
	],
	'menu.start': ['Go to the start', 'Aller au début', 'Ir al inicio', 'Zum Anfang'],
	'menu.end': ['Go to the end', 'Aller à la fin', 'Ir al final', 'Zum Ende'],
	'menu.scene.before': [
		'Insert a blank scene before',
		'Insérer une scène vide avant',
		'Insertar una escena vacía antes',
		'Leere Szene davor einfügen'
	],
	'menu.scene.after': [
		'Insert a blank scene after',
		'Insérer une scène vide après',
		'Insertar una escena vacía después',
		'Leere Szene danach einfügen'
	],
	'menu.scene.duplicate': ['Duplicate the scene', 'Dupliquer la scène', 'Duplicar la escena', 'Szene duplizieren'],
	'menu.scene.fade': [
		'Fade in from the scene before',
		'Fondu depuis la scène précédente',
		'Fundido desde la escena anterior',
		'Aus der vorigen Szene einblenden'
	],
	'menu.scene.cut': ['Remove the transition', 'Retirer la transition', 'Quitar la transición', 'Übergang entfernen'],
	'menu.scene.delete': ['Delete the scene', 'Supprimer la scène', 'Eliminar la escena', 'Szene löschen'],
	'menu.forward': ['Bring forward', 'Avancer', 'Traer adelante', 'Nach vorne'],
	'menu.backward': ['Send backward', 'Reculer', 'Enviar atrás', 'Nach hinten'],
	'menu.fill': ['Fill the frame', 'Remplir le cadre', 'Llenar el cuadro', 'Bild füllen'],
	'menu.fit': ['Fit in the frame', 'Ajuster au cadre', 'Ajustar al cuadro', 'Ins Bild einpassen'],
	'menu.centre': ['Centre on the stage', 'Centrer sur la scène', 'Centrar en el escenario', 'Auf der Bühne zentrieren'],
	'menu.reset': ['Reset the size', 'Réinitialiser la taille', 'Restablecer el tamaño', 'Größe zurücksetzen'],
	'menu.add': ['Add at the playhead', 'Ajouter à la tête de lecture', 'Añadir en el cabezal', 'Am Abspielkopf einfügen'],
	'menu.add.track': [
		'Add on a new track',
		'Ajouter sur une nouvelle piste',
		'Añadir en una pista nueva',
		'Auf neuer Spur einfügen'
	],
	'menu.preview': ['Preview', 'Aperçu', 'Vista previa', 'Vorschau'],
	'menu.rename': ['Rename', 'Renommer', 'Cambiar el nombre', 'Umbenennen'],
	'menu.marker.add': ['Add a marker here', 'Ajouter un repère ici', 'Añadir un marcador aquí', 'Hier einen Marker setzen'],
	'menu.marker.rename': ['Rename the marker', 'Renommer le repère', 'Cambiar el nombre del marcador', 'Marker umbenennen'],
	'menu.marker.delete': ['Delete the marker', 'Supprimer le repère', 'Eliminar el marcador', 'Marker löschen'],
	'help.open': ['Keyboard shortcuts', 'Raccourcis clavier', 'Atajos de teclado', 'Tastenkürzel'],
	'help.playback': ['Playback', 'Lecture', 'Reproducción', 'Wiedergabe'],
	'help.editing': ['Editing', 'Édition', 'Edición', 'Bearbeiten'],
	'help.menus': [
		'Right-click a clip, a scene, the ruler, the stage or a media file for more. On a keyboard: Shift+F10.',
		'Clic droit sur un clip, une scène, la règle, la scène ou un média pour plus d’actions. Au clavier : Maj+F10.',
		'Haga clic derecho en un clip, una escena, la regla, el escenario o un medio para más acciones. Con el teclado: Mayús+F10.',
		'Rechtsklick auf Clip, Szene, Lineal, Bühne oder Medium für weitere Aktionen. Per Tastatur: Umschalt+F10.'
	],
	'command.toggle': ['Play or pause', 'Lire ou mettre en pause', 'Reproducir o pausar', 'Abspielen oder pausieren'],
	'command.back': ['Play backwards, faster', 'Lire à rebours, plus vite', 'Reproducir hacia atrás, más rápido', 'Rückwärts, schneller'],
	'command.pause': ['Stop', 'Arrêter', 'Detener', 'Anhalten'],
	'command.forward': ['Play forwards, faster', 'Lire en avant, plus vite', 'Reproducir hacia delante, más rápido', 'Vorwärts, schneller'],
	'command.prevFrame': ['Previous frame', 'Image précédente', 'Fotograma anterior', 'Vorheriges Bild'],
	'command.nextFrame': ['Next frame', 'Image suivante', 'Fotograma siguiente', 'Nächstes Bild'],
	'command.prevSecond': ['One second back', 'Une seconde en arrière', 'Un segundo atrás', 'Eine Sekunde zurück'],
	'command.nextSecond': ['One second forward', 'Une seconde en avant', 'Un segundo adelante', 'Eine Sekunde vor'],
	'command.home': ['Go to the start', 'Aller au début', 'Ir al inicio', 'Zum Anfang'],
	'command.end': ['Go to the end', 'Aller à la fin', 'Ir al final', 'Zum Ende'],
	'command.split': ['Split at the playhead', 'Couper à la tête de lecture', 'Dividir en el cabezal', 'Am Abspielkopf teilen'],
	'command.delete': ['Delete the selection', 'Supprimer la sélection', 'Eliminar la selección', 'Auswahl löschen'],
	'command.ripple': [
		'Delete and close the gap',
		'Supprimer et refermer le vide',
		'Eliminar y cerrar el hueco',
		'Löschen und Lücke schließen'
	],
	'command.cut': ['Cut the selected clip', 'Couper le clip sélectionné', 'Cortar el clip seleccionado', 'Ausgewählten Clip ausschneiden'],
	'command.marker': [
		'Add a marker at the playhead',
		'Ajouter un repère à la tête de lecture',
		'Añadir un marcador en el cabezal',
		'Marker am Abspielkopf setzen'
	],
	'command.copy': ['Copy the selected clip', 'Copier le clip sélectionné', 'Copiar el clip seleccionado', 'Ausgewählten Clip kopieren'],
	'command.paste': ['Paste at the playhead', 'Coller à la tête de lecture', 'Pegar en el cabezal', 'Am Abspielkopf einfügen'],
	'command.duplicate': ['Duplicate the selection', 'Dupliquer la sélection', 'Duplicar la selección', 'Auswahl duplizieren'],
	'command.undo': ['Undo', 'Annuler', 'Deshacer', 'Rückgängig'],
	'command.redo': ['Redo', 'Rétablir', 'Rehacer', 'Wiederholen'],
	'command.help': ['Show the shortcuts', 'Afficher les raccourcis', 'Mostrar los atajos', 'Tastenkürzel zeigen']
} satisfies Record<string, Row>;
