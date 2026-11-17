import type { Row } from '../row';

/** The stage overlay, the clip inspector and the transition pickers. */
export const stage = {
	'editor.clip': ['Clip', 'Clip', 'Clip', 'Clip'],
	'stage.label': [
		'Stage. Drag a clip to move it, arrows nudge it.',
		'Scène. Glissez un clip pour le déplacer, les flèches l’ajustent.',
		'Escenario. Arrastre un clip para moverlo, las flechas lo ajustan.',
		'Bühne. Clip ziehen zum Verschieben, Pfeiltasten zum Feinjustieren.'
	],
	'stage.clip': ['{kind} {id}', '{kind} {id}', '{kind} {id}', '{kind} {id}'],
	'inspector.title': ['Clip settings', 'Réglages du clip', 'Ajustes del clip', 'Clip-Einstellungen'],
	'inspector.empty': [
		'Select a clip on the stage or in the timeline to set its place, look and motion.',
		'Sélectionnez un clip sur la scène ou dans la timeline pour régler sa place, son aspect et son animation.',
		'Seleccione un clip en el escenario o en la línea de tiempo para ajustar su posición, aspecto y animación.',
		'Wählen Sie einen Clip auf der Bühne oder in der Timeline, um Position, Aussehen und Animation einzustellen.'
	],
	'inspector.none.title': ['No clip selected', 'Aucun clip sélectionné', 'Ningún clip seleccionado', 'Kein Clip ausgewählt'],
	'inspector.text': ['Text', 'Texte', 'Texto', 'Text'],
	'inspector.width': ['Width', 'Largeur', 'Ancho', 'Breite'],
	'inspector.height': ['Height', 'Hauteur', 'Alto', 'Höhe'],
	'inspector.rotation': ['Rotation', 'Rotation', 'Rotación', 'Drehung'],
	'inspector.opacity': ['Opacity', 'Opacité', 'Opacidad', 'Deckkraft'],
	'inspector.volume': ['Volume', 'Volume', 'Volumen', 'Lautstärke'],
	'inspector.start': ['Start', 'Début', 'Inicio', 'Start'],
	'inspector.duration': ['Length', 'Durée', 'Duración', 'Länge'],
	'inspector.seconds': ['Length', 'Durée', 'Duración', 'Länge'],
	'inspector.fill': ['Fill the stage', 'Remplir la scène', 'Llenar el escenario', 'Bühne füllen'],
	'inspector.centre': ['Centre', 'Centrer', 'Centrar', 'Zentrieren'],
	'inspector.fit': ['Fit', 'Ajustement', 'Ajuste', 'Einpassen'],
	'inspector.fit.cover': ['Cover the box', 'Couvrir le cadre', 'Cubrir el marco', 'Rahmen füllen'],
	'inspector.fit.contain': ['Show it whole', 'Montrer en entier', 'Mostrar entero', 'Ganz zeigen'],
	'inspector.fit.fill': ['Stretch', 'Étirer', 'Estirar', 'Strecken'],
	'inspector.enter': ['Enter', 'Entrée', 'Entrada', 'Eingang'],
	'inspector.exit': ['Exit', 'Sortie', 'Salida', 'Ausgang'],
	'inspector.none': ['None', 'Aucune', 'Ninguna', 'Keine'],
	'transition.fade': ['Fade', 'Fondu', 'Fundido', 'Überblenden'],
	'transition.dip': ['Dip to black', 'Passage au noir', 'Paso a negro', 'Über Schwarz'],
	'transition.slide-left': ['Slide left', 'Glisser à gauche', 'Deslizar a la izquierda', 'Nach links schieben'],
	'transition.slide-right': ['Slide right', 'Glisser à droite', 'Deslizar a la derecha', 'Nach rechts schieben'],
	'transition.slide-up': ['Slide up', 'Glisser vers le haut', 'Deslizar hacia arriba', 'Nach oben schieben'],
	'transition.slide-down': ['Slide down', 'Glisser vers le bas', 'Deslizar hacia abajo', 'Nach unten schieben'],
	'transition.zoom': ['Zoom', 'Zoom', 'Zoom', 'Zoom'],
	'transition.zoom-in': ['Zoom in', 'Zoom avant', 'Acercar', 'Hineinzoomen'],
	'transition.zoom-out': ['Zoom out', 'Zoom arrière', 'Alejar', 'Herauszoomen'],
	'transition.wipe-left': ['Wipe left', 'Volet à gauche', 'Cortinilla a la izquierda', 'Wischen nach links'],
	'transition.wipe-right': ['Wipe right', 'Volet à droite', 'Cortinilla a la derecha', 'Wischen nach rechts'],
	'transition.blur': ['Blur', 'Flou', 'Desenfoque', 'Unschärfe'],
	'transition.pop': ['Pop', 'Rebond', 'Rebote', 'Aufploppen'],
	'transition.scene': [
		'Transition into scene {scene}: {name}',
		'Transition vers la scène {scene} : {name}',
		'Transición a la escena {scene}: {name}',
		'Übergang zu Szene {scene}: {name}'
	],
	'transition.length': ['Length in seconds', 'Durée en secondes', 'Duración en segundos', 'Länge in Sekunden']
} satisfies Record<string, Row>;
