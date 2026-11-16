import type { Row } from '../row';

/** The render dialog and the renders history. */
export const render = {
	'render.title': ['Render', 'Rendu', 'Renderizar', 'Rendern'],
	'render.open': ['Render', 'Rendre', 'Renderizar', 'Rendern'],
	'render.resolution': ['Resolution', 'Résolution', 'Resolución', 'Auflösung'],
	'render.fps': ['Frames per second', 'Images par seconde', 'Fotogramas por segundo', 'Bilder pro Sekunde'],
	'render.range': ['Range', 'Plage', 'Intervalo', 'Bereich'],
	'render.range.all': ['Whole project', 'Projet entier', 'Proyecto completo', 'Ganzes Projekt'],
	'render.range.part': ['From in to out', 'Entre deux points', 'Entre dos puntos', 'Von In bis Out'],
	'render.in': ['In (seconds)', 'Début (secondes)', 'Inicio (segundos)', 'In (Sekunden)'],
	'render.out': ['Out (seconds)', 'Fin (secondes)', 'Fin (segundos)', 'Out (Sekunden)'],
	'render.start': ['Start the render', 'Lancer le rendu', 'Iniciar el render', 'Rendering starten'],
	'render.cancel': ['Cancel the render', 'Annuler le rendu', 'Cancelar el render', 'Rendering abbrechen'],
	'render.close': ['Close', 'Fermer', 'Cerrar', 'Schließen'],
	'render.again': ['New render', 'Nouveau rendu', 'Nuevo render', 'Neues Rendering'],
	'render.download': ['Download the MP4', 'Télécharger le MP4', 'Descargar el MP4', 'MP4 herunterladen'],
	'render.progress': ['Frame {frame} of {frames}', 'Image {frame} sur {frames}', 'Fotograma {frame} de {frames}', 'Bild {frame} von {frames}'],
	'render.limits': [
		'Up to 1080p, 30 frames per second and 10 minutes. One render runs at a time.',
		'Jusqu’à 1080p, 30 images par seconde et 10 minutes. Un seul rendu tourne à la fois.',
		'Hasta 1080p, 30 fotogramas por segundo y 10 minutos. Solo se ejecuta un render a la vez.',
		'Bis 1080p, 30 Bilder pro Sekunde und 10 Minuten. Es läuft immer nur ein Rendering.'
	],
	'render.failed': ['The render could not start', 'Le rendu n’a pas pu démarrer', 'No se pudo iniciar el render', 'Das Rendering konnte nicht starten'],
	'render.history': ['Past renders', 'Rendus précédents', 'Renders anteriores', 'Frühere Renderings'],
	'render.history.empty': ['No render yet', 'Aucun rendu', 'Ningún render', 'Noch kein Rendering'],
	'render.state.queued': ['Queued', 'En attente', 'En cola', 'In Warteschlange'],
	'render.state.running': ['Rendering', 'En cours', 'Renderizando', 'Wird gerendert'],
	'render.state.done': ['Done', 'Terminé', 'Terminado', 'Fertig'],
	'render.state.failed': ['Failed', 'Échec', 'Fallido', 'Fehlgeschlagen'],
	'render.state.cancelled': ['Cancelled', 'Annulé', 'Cancelado', 'Abgebrochen']
} satisfies Record<string, Row>;
