import type { Row } from '../row';

/** The page where a person confirms or refuses an action an agent asked for. */
export const pending = {
	'pending.title': ['Agent request', 'Demande d’un agent', 'Solicitud de un agente', 'Anfrage eines Agenten'],
	'pending.subtitle': [
		'An agent wants to make a change that needs your approval.',
		'Un agent veut faire une modification qui demande votre accord.',
		'Un agente quiere hacer un cambio que requiere su aprobación.',
		'Ein Agent möchte eine Änderung vornehmen, die Ihre Zustimmung braucht.'
	],
	'pending.ready': ['Ready', 'Prête', 'Lista', 'Bereit'],
	'pending.changed': [
		'The project changed since the request: confirming will fail',
		'Le projet a changé depuis la demande : la confirmation échouera',
		'El proyecto cambió desde la solicitud: la confirmación fallará',
		'Das Projekt wurde seit der Anfrage geändert: Die Bestätigung schlägt fehl'
	],
	'pending.gone': [
		'The project no longer exists, or you no longer have access to it',
		'Le projet n’existe plus ou vous n’y avez plus accès',
		'El proyecto ya no existe o ya no tiene acceso a él',
		'Das Projekt existiert nicht mehr, oder Sie haben keinen Zugriff mehr'
	],
	'pending.client': ['Client {client}', 'Client {client}', 'Cliente {client}', 'Client {client}'],
	'pending.expires': ['Expires {moment}', 'Expire le {moment}', 'Caduca el {moment}', 'Läuft ab am {moment}'],
	'pending.open': ['Open the project', 'Ouvrir le projet', 'Abrir el proyecto', 'Projekt öffnen'],
	'pending.confirmed': ['Request confirmed', 'Demande confirmée', 'Solicitud confirmada', 'Anfrage bestätigt'],
	'pending.confirmed.hint': [
		'The project is in the trash, where it stays 30 days.',
		'Le projet est dans la corbeille, où il reste 30 jours.',
		'El proyecto está en la papelera, donde permanece 30 días.',
		'Das Projekt liegt im Papierkorb und bleibt dort 30 Tage.'
	],
	'pending.dismissed': ['Request refused', 'Demande refusée', 'Solicitud rechazada', 'Anfrage abgelehnt'],
	'pending.dismissed.hint': [
		'The project stays as it is.',
		'Le projet reste tel quel.',
		'El proyecto queda como está.',
		'Das Projekt bleibt, wie sie ist.'
	],
	'pending.missing': ['Request not found', 'Demande introuvable', 'Solicitud no encontrada', 'Anfrage nicht gefunden'],
	'pending.missing.hint': [
		'It was already confirmed, refused or expired, or it is not in your name.',
		'Elle a déjà été confirmée, refusée ou a expiré, ou elle n’est pas à votre nom.',
		'Ya fue confirmada, rechazada o caducó, o no está a su nombre.',
		'Sie wurde bereits bestätigt, abgelehnt oder ist abgelaufen, oder sie läuft nicht auf Ihren Namen.'
	],
	'pending.truncated': [
		'Too many open requests',
		'Trop de demandes ouvertes',
		'Demasiadas solicitudes abiertas',
		'Zu viele offene Anfragen'
	],
	'pending.truncated.hint': [
		'This one is not among the 100 most recent. Answer the most recent first, then open this link again.',
		'Celle-ci n’est pas parmi les 100 plus récentes. Répondez d’abord aux plus récentes, puis rouvrez ce lien.',
		'Esta no está entre las 100 más recientes. Responda primero a las más recientes y vuelva a abrir este enlace.',
		'Diese gehört nicht zu den 100 neuesten. Beantworten Sie zuerst die neuesten und öffnen Sie den Link erneut.'
	],
	'pending.browser': [
		'Sign in again in your browser',
		'Reconnectez-vous dans votre navigateur',
		'Vuelva a iniciar sesión en su navegador',
		'Melden Sie sich im Browser erneut an'
	],
	'pending.browser.hint': [
		'Only a session opened in your browser can confirm an agent request.',
		'Seule une session ouverte dans votre navigateur peut confirmer une demande d’agent.',
		'Solo una sesión abierta en su navegador puede confirmar una solicitud de agente.',
		'Nur eine im Browser geöffnete Sitzung kann eine Agentenanfrage bestätigen.'
	],
	'pending.signin': ['Sign in again', 'Se reconnecter', 'Volver a iniciar sesión', 'Erneut anmelden']
} satisfies Record<string, Row>;
