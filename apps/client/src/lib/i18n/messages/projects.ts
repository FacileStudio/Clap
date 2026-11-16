import type { Row } from '../row';

/** The shell, the project list, the settings page and the error page. */
export const projects = {
	account: ['Account', 'Compte', 'Cuenta', 'Konto'],
	'action.cancel': ['Cancel', 'Annuler', 'Cancelar', 'Abbrechen'],
	'action.create': ['Create', 'Créer', 'Crear', 'Erstellen'],
	'action.retry': ['Retry', 'Réessayer', 'Reintentar', 'Erneut versuchen'],
	'action.save': ['Save', 'Enregistrer', 'Guardar', 'Speichern'],
	'action.undo': ['Undo', 'Annuler', 'Deshacer', 'Rückgängig'],
	'projects.title': ['Projects', 'Projets', 'Proyectos', 'Projekte'],
	'projects.subtitle': [
		'Each project is one text file: scenes, clips and titles.',
		'Chaque projet est un seul fichier texte : scènes, clips et titres.',
		'Cada proyecto es un solo archivo de texto: escenas, clips y títulos.',
		'Jedes Projekt ist eine einzige Textdatei: Szenen, Clips und Titel.'
	],
	'projects.new': ['New project', 'Nouveau projet', 'Nuevo proyecto', 'Neues Projekt'],
	'projects.empty': ['No project yet', 'Aucun projet', 'Ningún proyecto', 'Noch kein Projekt'],
	'projects.empty.hint': [
		'Create one, drop your clips on the timeline and render an MP4.',
		'Créez-en un, déposez vos clips sur la timeline et rendez un MP4.',
		'Cree uno, suelte sus clips en la línea de tiempo y renderice un MP4.',
		'Erstellen Sie eines, legen Sie Ihre Clips auf die Timeline und rendern Sie ein MP4.'
	],
	'projects.failed': [
		'The projects could not be loaded.',
		'Les projets n’ont pas pu être chargés.',
		'No se pudieron cargar los proyectos.',
		'Die Projekte konnten nicht geladen werden.'
	],
	'project.title': ['Title', 'Titre', 'Título', 'Titel'],
	'project.untitled': ['Untitled', 'Sans titre', 'Sin título', 'Ohne Titel'],
	'project.rename': ['Rename', 'Renommer', 'Renombrar', 'Umbenennen'],
	'project.trash': ['Move to trash', 'Mettre à la corbeille', 'Mover a la papelera', 'In den Papierkorb'],
	'project.trashed': [
		'“{title}” moved to the trash',
		'« {title} » mis à la corbeille',
		'«{title}» movido a la papelera',
		'„{title}“ in den Papierkorb verschoben'
	],
	'project.actions': ['Actions for {title}', 'Actions pour {title}', 'Acciones para {title}', 'Aktionen für {title}'],
	'project.missing': [
		'Project not found',
		'Projet introuvable',
		'Proyecto no encontrado',
		'Projekt nicht gefunden'
	],
	'project.missing.hint': [
		'It was deleted, or you do not have access to it.',
		'Il a été supprimé, ou vous n’y avez pas accès.',
		'Fue eliminado, o no tiene acceso a él.',
		'Es wurde gelöscht, oder Sie haben keinen Zugriff darauf.'
	],
	'project.back': [
		'Back to projects',
		'Retour aux projets',
		'Volver a los proyectos',
		'Zurück zu den Projekten'
	],
	'trash.title': ['Trash', 'Corbeille', 'Papelera', 'Papierkorb'],
	'trash.subtitle': [
		'Trashed projects. Restore one, or delete it for good.',
		'Les projets mis à la corbeille. Restaurez-en un, ou supprimez-le définitivement.',
		'Los proyectos en la papelera. Restaure uno, o elimínelo definitivamente.',
		'Projekte im Papierkorb. Stellen Sie eines wieder her oder löschen Sie es endgültig.'
	],
	'trash.empty': ['The trash is empty', 'La corbeille est vide', 'La papelera está vacía', 'Der Papierkorb ist leer'],
	'trash.restore': ['Restore', 'Restaurer', 'Restaurar', 'Wiederherstellen'],
	'trash.delete': ['Delete for good', 'Supprimer définitivement', 'Eliminar definitivamente', 'Endgültig löschen'],
	'trash.confirm': [
		'Delete “{title}” for good?',
		'Supprimer définitivement « {title} » ?',
		'¿Eliminar definitivamente «{title}»?',
		'„{title}“ endgültig löschen?'
	],
	'trash.confirm.hint': [
		'The project, its history and its files are deleted. This cannot be undone.',
		'Le projet, son historique et ses fichiers sont supprimés. Cette action est irréversible.',
		'El proyecto, su historial y sus archivos se eliminan. Esta acción no se puede deshacer.',
		'Das Projekt, sein Verlauf und seine Dateien werden gelöscht. Das lässt sich nicht rückgängig machen.'
	],
	'settings.title': ['Settings', 'Réglages', 'Ajustes', 'Einstellungen'],
	'settings.profile': ['Profile', 'Profil', 'Perfil', 'Profil'],
	'settings.preferences': ['Preferences', 'Préférences', 'Preferencias', 'Einstellungen'],
	'error.title': ['Error', 'Erreur', 'Error', 'Fehler'],
	'error.generic': [
		'Something went wrong',
		'Une erreur est survenue',
		'Se produjo un error',
		'Ein Fehler ist aufgetreten'
	],
	'error.hint': [
		'Reload the page. If the error comes back, try again later.',
		'Rechargez la page. Si l’erreur revient, réessayez plus tard.',
		'Recargue la página. Si el error vuelve, inténtelo más tarde.',
		'Laden Sie die Seite neu. Tritt der Fehler erneut auf, versuchen Sie es später.'
	],
	'error.missing': ['Page not found', 'Page introuvable', 'Página no encontrada', 'Seite nicht gefunden'],
	'error.missing.hint': [
		'This address leads to no page of Clap.',
		'Cette adresse ne mène à aucune page de Clap.',
		'Esta dirección no lleva a ninguna página de Clap.',
		'Diese Adresse führt zu keiner Seite von Clap.'
	]
} satisfies Record<string, Row>;
