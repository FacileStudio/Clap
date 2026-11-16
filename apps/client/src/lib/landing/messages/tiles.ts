import type { Row } from '@facile/muse';

/** The five feature tiles, in the order en, fr, es, de: a title, one line, and the words drawn inside each demo. */
export const tiles = {
	'tile.file.title': [
		'A video is one text file',
		'Une vidéo, c’est un fichier texte',
		'Un vídeo es un archivo de texto',
		'Ein Video ist eine Textdatei'
	],
	'tile.file.body': [
		'Change a line and the preview follows. The file stays readable from top to bottom.',
		'Changez une ligne, l’aperçu suit. Le fichier reste lisible de haut en bas.',
		'Cambie una línea y la vista previa la sigue. El archivo se lee de arriba abajo.',
		'Ändern Sie eine Zeile, die Vorschau folgt. Die Datei bleibt von oben bis unten lesbar.'
	],
	'tile.file.before': ['Summer', 'Été', 'Verano', 'Sommer'],
	'tile.bulle.title': [
		'Bulle does the motion design',
		'Bulle fait le motion design',
		'Bulle hace el motion design',
		'Bulle übernimmt das Motion Design'
	],
	'tile.bulle.body': [
		'Ask for an animated title and the scene lands in your project. Bulle needs a plan with AI.',
		'Demandez un titre animé, la scène arrive dans votre projet. Bulle demande une offre avec l’IA.',
		'Pida un título animado y la escena llega a su proyecto. Bulle requiere un plan con IA.',
		'Fragen Sie nach einem animierten Titel, die Szene landet im Projekt. Bulle braucht einen Tarif mit KI.'
	],
	'tile.bulle.ask': [
		'Add an animated title: Brittany, August',
		'Ajoute un titre animé : Bretagne, août',
		'Añade un título animado: Bretaña, agosto',
		'Füge einen animierten Titel hinzu: Bretagne, August'
	],
	'tile.bulle.done': ['Scene added', 'Scène ajoutée', 'Escena añadida', 'Szene hinzugefügt'],
	'tile.bulle.placeholder': ['Ask Bulle', 'Demander à Bulle', 'Preguntar a Bulle', 'Bulle fragen'],
	'tile.timeline.title': ['A real timeline', 'Une vraie timeline', 'Una línea de tiempo de verdad', 'Eine echte Timeline'],
	'tile.timeline.body': [
		'Drag, trim and split your clips. They snap to each other.',
		'Glissez, rognez et coupez vos clips. Ils s’aimantent entre eux.',
		'Arrastre, recorte y divida sus clips. Se ajustan entre sí.',
		'Clips ziehen, kürzen und teilen. Sie rasten aneinander ein.'
	],
	'tile.render.title': ['Rendered on our servers', 'Rendu sur nos serveurs', 'Renderizado en nuestros servidores', 'Auf unseren Servern gerendert'],
	'tile.render.body': [
		'Start the render and follow it frame by frame. You get an MP4, up to 1080p.',
		'Lancez le rendu et suivez-le image par image. Vous récupérez un MP4, jusqu’en 1080p.',
		'Inicie el render y sígalo fotograma a fotograma. Recibe un MP4, hasta 1080p.',
		'Starten Sie das Rendering und verfolgen Sie es Bild für Bild. Sie erhalten ein MP4, bis 1080p.'
	],
	'tile.cli.title': ['From the terminal', 'Depuis le terminal', 'Desde la terminal', 'Aus dem Terminal'],
	'tile.cli.body': [
		'The clap command creates, previews and renders a project on your machine.',
		'La commande clap crée, prévisualise et rend un projet sur votre machine.',
		'El comando clap crea, previsualiza y renderiza un proyecto en su máquina.',
		'Der Befehl clap erstellt, zeigt und rendert ein Projekt auf Ihrem Rechner.'
	],
	'tile.cli.project': ['summer', 'ete', 'verano', 'sommer']
} satisfies Record<string, Row>;
