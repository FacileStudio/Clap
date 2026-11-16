import type { Row } from '../row';

/** The share dialog and the public player. */
export const share = {
	'share.open': ['Share', 'Partager', 'Compartir', 'Teilen'],
	'share.title': ['Share', 'Partager', 'Compartir', 'Teilen'],
	'share.failed': [
		'The share state could not be read.',
		'L’état du partage n’a pas pu être lu.',
		'No se pudo leer el estado del enlace.',
		'Der Freigabestatus konnte nicht gelesen werden.'
	],
	'share.off': [
		'Create a link, and anyone who has it can watch the latest render of this project in a browser, without an account. They never see your source or your media.',
		'Créez un lien : toute personne qui l’a peut regarder le dernier rendu de ce projet dans un navigateur, sans compte. Elle ne voit jamais votre source ni vos médias.',
		'Crea un enlace: quien lo tenga puede ver el último render de este proyecto en un navegador, sin cuenta. Nunca ve tu fuente ni tus medios.',
		'Erstellen Sie einen Link: Wer ihn hat, kann das letzte Rendering dieses Projekts im Browser ansehen, ohne Konto. Quelltext und Medien bleiben verborgen.'
	],
	'share.norender': [
		'This project has no finished render yet. A link shows the latest one, so it shows nothing until a render is done.',
		'Ce projet n’a pas encore de rendu terminé. Un lien montre le dernier : il ne montre rien tant qu’aucun rendu n’est fini.',
		'Este proyecto aún no tiene un render terminado. Un enlace muestra el último, así que no muestra nada hasta que uno termine.',
		'Dieses Projekt hat noch kein fertiges Rendering. Ein Link zeigt das letzte, also zeigt er nichts, bis eines fertig ist.'
	],
	'share.render': ['Start a render', 'Lancer un rendu', 'Iniciar un render', 'Rendering starten'],
	'share.create': ['Create the link', 'Créer le lien', 'Crear el enlace', 'Link erstellen'],
	'share.once': ['Copy this link now', 'Copiez ce lien maintenant', 'Copia este enlace ahora', 'Kopieren Sie diesen Link jetzt'],
	'share.once.hint': [
		'It is shown only this once. Clap does not keep it and cannot show it again. If you lose it, generate a new one.',
		'Il n’est affiché que cette fois. Clap ne le conserve pas et ne pourra pas vous le remontrer. Si vous le perdez, générez-en un nouveau.',
		'Solo se muestra esta vez. Clap no lo guarda y no podrá mostrarlo de nuevo. Si lo pierdes, genera uno nuevo.',
		'Er wird nur dieses eine Mal angezeigt. Clap speichert ihn nicht und kann ihn nicht erneut zeigen. Geht er verloren, erzeugen Sie einen neuen.'
	],
	'share.link': ['Link', 'Lien', 'Enlace', 'Link'],
	'share.copy': ['Copy the link', 'Copier le lien', 'Copiar el enlace', 'Link kopieren'],
	'share.embed': ['Embed code', 'Code d’intégration', 'Código de inserción', 'Einbettungscode'],
	'share.embed.hint': [
		'Paste it into a page to play the video there.',
		'Collez-le dans une page pour y lire la vidéo.',
		'Pégalo en una página para reproducir allí el vídeo.',
		'Fügen Sie ihn in eine Seite ein, um das Video dort abzuspielen.'
	],
	'share.embed.copy': ['Copy the embed code', 'Copier le code d’intégration', 'Copiar el código de inserción', 'Einbettungscode kopieren'],
	'share.on': [
		'This project is shared since {moment}.',
		'Ce projet est partagé depuis le {moment}.',
		'Este proyecto está compartido desde el {moment}.',
		'Dieses Projekt ist seit {moment} freigegeben.'
	],
	'share.hidden': [
		'The link was shown when it was created and cannot be shown again. Generate a new one if you no longer have it.',
		'Le lien a été affiché à sa création et ne peut pas être remontré. Générez-en un nouveau si vous ne l’avez plus.',
		'El enlace se mostró al crearlo y no puede mostrarse de nuevo. Genera uno nuevo si ya no lo tienes.',
		'Der Link wurde bei der Erstellung angezeigt und kann nicht erneut gezeigt werden. Erzeugen Sie einen neuen, wenn Sie ihn nicht mehr haben.'
	],
	'share.rotate': ['Generate a new link', 'Générer un nouveau lien', 'Generar un enlace nuevo', 'Neuen Link erzeugen'],
	'share.rotate.title': ['Replace the link?', 'Remplacer le lien ?', '¿Reemplazar el enlace?', 'Link ersetzen?'],
	'share.rotate.hint': [
		'The current link stops working at once, wherever it was sent or embedded.',
		'Le lien actuel cesse de fonctionner tout de suite, partout où il a été envoyé ou intégré.',
		'El enlace actual deja de funcionar de inmediato, dondequiera que se haya enviado o insertado.',
		'Der aktuelle Link funktioniert sofort nicht mehr, überall, wo er verschickt oder eingebettet wurde.'
	],
	'share.revoke': ['Turn the link off', 'Désactiver le lien', 'Desactivar el enlace', 'Link deaktivieren'],
	'share.revoke.title': ['Turn the link off?', 'Désactiver le lien ?', '¿Desactivar el enlace?', 'Link deaktivieren?'],
	'share.revoke.hint': [
		'Nobody can watch the video through it any more, wherever it was sent or embedded. A new link can be created later.',
		'Plus personne ne peut regarder la vidéo avec ce lien, partout où il a été envoyé ou intégré. Vous pourrez en créer un nouveau.',
		'Nadie podrá ver el vídeo con este enlace, dondequiera que se haya enviado o insertado. Podrá crear uno nuevo más tarde.',
		'Niemand kann das Video mehr über diesen Link ansehen, überall, wo er verschickt oder eingebettet wurde. Später lässt sich ein neuer erstellen.'
	],
	'public.gone': [
		'This video is no longer available',
		'Cette vidéo n’est plus disponible',
		'Este vídeo ya no está disponible',
		'Dieses Video ist nicht mehr verfügbar'
	],
	'public.gone.hint': [
		'Its owner turned the link off or replaced it. Ask them for a new link.',
		'Son auteur a désactivé ou remplacé le lien. Demandez-lui un nouveau lien.',
		'Su autor desactivó o reemplazó el enlace. Pídele un enlace nuevo.',
		'Der Besitzer hat den Link deaktiviert oder ersetzt. Bitten Sie um einen neuen Link.'
	],
	'public.made': ['Made with Clap', 'Fait avec Clap', 'Hecho con Clap', 'Erstellt mit Clap'],
	'public.open': ['Open in a new tab', 'Ouvrir dans un nouvel onglet', 'Abrir en una pestaña nueva', 'In neuem Tab öffnen']
} satisfies Record<string, Row>;
