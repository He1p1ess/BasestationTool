use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    De,
    En,
    Fr,
    Es,
    It,
}
impl Language {
    pub const ALL: [Self; 5] = [Self::De, Self::En, Self::Fr, Self::Es, Self::It];
    pub fn name(self) -> &'static str {
        self.text(["Deutsch", "English", "Français", "Español", "Italiano"])
    }
    pub fn text(self, values: [&'static str; 5]) -> &'static str {
        values[self as usize]
    }
}
pub mod strings {
    pub const IDENTIFY: [&str; 5] = [
        "Identifizieren",
        "Identify",
        "Identifier",
        "Identificar",
        "Identifica",
    ];
    pub const SPINNING_UP: [&str; 5] = [
        "Fährt hoch",
        "Starting up",
        "Démarrage en cours",
        "Arrancando",
        "Avvio in corso",
    ];
    pub const RUNNING: [&str; 5] = ["Aktiv", "Running", "Actif", "Activo", "Attivo"];
    pub const STOPPED: [&str; 5] = ["Inaktiv", "Stopped", "Inactif", "Inactivo", "Inattivo"];
    pub const EXPORTED: [&str; 5] = [
        "Exportiert",
        "Exported",
        "Exporté",
        "Exportado",
        "Esportato",
    ];
    pub const COULD_NOT_OPEN_FOLDER: [&str; 5] = [
        "Ordner konnte nicht geöffnet werden",
        "Could not open folder",
        "Impossible d’ouvrir le dossier",
        "No se pudo abrir la carpeta",
        "Impossibile aprire la cartella",
    ];
    pub const STEAMVR_AUTOSTART_VERIFIED: [&str; 5] = [
        "SteamVR-Autostart bestätigt",
        "SteamVR autostart verified",
        "Démarrage avec SteamVR confirmé",
        "Inicio con SteamVR confirmado",
        "Avvio con SteamVR confermato",
    ];
    pub const NO_STATIONS_SELECTED_CLICK_A_NAME_OR_TICK_STATIONS: [&str; 5] = [
        "Keine Station ausgewählt. Namen anklicken oder Stationen anhaken.",
        "No stations selected. Click a name or tick stations.",
        "Aucune station sélectionnée. Cliquez sur un nom ou cochez des stations.",
        "Ninguna estación seleccionada. Haz clic en un nombre o marca las estaciones.",
        "Nessuna stazione selezionata. Fai clic su un nome o spunta le stazioni.",
    ];
    pub const SELECTED: [&str; 5] = ["Auswahl", "Selected", "Sélection", "Selección", "Selezione"];
    pub const ON: [&str; 5] = ["Ein", "On", "Allumer", "Encender", "Accendi"];
    pub const STANDBY: [&str; 5] = ["Standby", "Standby", "Veille", "En espera", "Standby"];
    pub const SLEEP: [&str; 5] = [
        "Sleep",
        "Sleep",
        "Veille profonde",
        "Suspensión",
        "Sospensione",
    ];
    pub const REFRESH_STATUS: [&str; 5] = [
        "Status abfragen",
        "Refresh status",
        "Actualiser l’état",
        "Actualizar estado",
        "Aggiorna stato",
    ];
    pub const SETTINGS: [&str; 5] = [
        "Einstellungen",
        "Settings",
        "Paramètres",
        "Configuración",
        "Impostazioni",
    ];
    pub const LANGUAGE: [&str; 5] = ["Sprache", "Language", "Langue", "Idioma", "Lingua"];
    pub const SHOW_LOG: [&str; 5] = [
        "Log anzeigen",
        "Show log",
        "Afficher le journal",
        "Mostrar registro",
        "Mostra registro",
    ];
    pub const APPEARANCE: [&str; 5] = [
        "Darstellung",
        "Appearance",
        "Apparence",
        "Apariencia",
        "Aspetto",
    ];
    pub const SYSTEM: [&str; 5] = ["System", "System", "Système", "Sistema", "Sistema"];
    pub const DARK: [&str; 5] = ["Dunkel", "Dark", "Sombre", "Oscuro", "Scuro"];
    pub const LIGHT: [&str; 5] = ["Hell", "Light", "Clair", "Claro", "Chiaro"];
    pub const AUTOMATICALLY_SCAN_EVERY_60_SECONDS: [&str; 5] = [
        "Automatisch suchen (alle 60 Sekunden)",
        "Automatically scan (every 60 seconds)",
        "Rechercher automatiquement (toutes les 60 secondes)",
        "Buscar automáticamente (cada 60 segundos)",
        "Cerca automaticamente (ogni 60 secondi)",
    ];
    pub const AUTOMATICALLY_SAVE_LOGS: [&str; 5] = [
        "Logs automatisch speichern",
        "Automatically save logs",
        "Enregistrer automatiquement les journaux",
        "Guardar los registros automáticamente",
        "Salva automaticamente i registri",
    ];
    pub const BLE_DEBUG_OUTPUT: [&str; 5] = [
        "BLE-Debug-Ausgabe",
        "BLE debug output",
        "Messages de débogage BLE",
        "Mensajes de depuración BLE",
        "Messaggi di debug BLE",
    ];
    pub const START_STEAMVR_BEFORE_CHANGING_REGISTRATION: [&str; 5] = [
        "SteamVR zum Ändern der Registrierung starten.",
        "Start SteamVR before changing registration.",
        "Démarrez SteamVR avant de modifier l’enregistrement.",
        "Inicia SteamVR antes de cambiar el registro.",
        "Avvia SteamVR prima di modificare la registrazione.",
    ];
    pub const AUTOMATICALLY_START_WITH_STEAMVR: [&str; 5] = [
        "Automatisch mit SteamVR starten",
        "Automatically start with SteamVR",
        "Démarrer automatiquement avec SteamVR",
        "Iniciar automáticamente con SteamVR",
        "Avvia automaticamente con SteamVR",
    ];
    pub const OPTIONAL_ABSOLUTE_PATH_TO_THE_OFFICIAL_OPENVR_LIBRARY: [&str; 5] = [
        "Optional: absoluter Pfad zur offiziellen OpenVR-Bibliothek",
        "Optional: absolute path to the official OpenVR library",
        "Facultatif : chemin absolu de la bibliothèque OpenVR officielle",
        "Opcional: ruta absoluta a la biblioteca oficial de OpenVR",
        "Facoltativo: percorso assoluto della libreria OpenVR ufficiale",
    ];
    pub const CLOSE_WITH_STEAMVR_ONLY_WHEN_LAUNCHED_BY_STEAMVR: [&str; 5] = [
        "Mit SteamVR beenden (nur bei SteamVR-Start)",
        "Close with SteamVR (only when launched by SteamVR)",
        "Fermer avec SteamVR (si lancé par SteamVR)",
        "Cerrar con SteamVR (solo si SteamVR inició la aplicación)",
        "Chiudi con SteamVR (solo se avviato da SteamVR)",
    ];
    pub const THESE_OPTIONS_AUTHORIZE_REAL_AUTOMATIC_POWER_COMMANDS_TO_ALL_KNOWN_STATIONS: [&str;
        5] = [
        "Diese Optionen erlauben echte automatische Power-Befehle an alle bekannten Stationen.",
        "These options authorize real automatic power commands to all known stations.",
        "Ces options autorisent l’envoi automatique de commandes d’alimentation à toutes les stations connues.",
        "Estas opciones autorizan el envío automático de órdenes de alimentación a todas las estaciones conocidas.",
        "Queste opzioni autorizzano l’invio automatico di comandi di alimentazione a tutte le stazioni note.",
    ];
    pub const POWER_ON_WHEN_STEAMVR_STARTS: [&str; 5] = [
        "Bei SteamVR-Start einschalten",
        "Power on when SteamVR starts",
        "Allumer les stations au démarrage de SteamVR",
        "Encender las estaciones al iniciar SteamVR",
        "Accendi le stazioni all’avvio di SteamVR",
    ];
    pub const WHEN_STEAMVR_EXITS: [&str; 5] = [
        "Beim SteamVR-Ende:",
        "When SteamVR exits:",
        "À la fermeture de SteamVR :",
        "Al cerrar SteamVR:",
        "Alla chiusura di SteamVR:",
    ];
    pub const NO_ACTION: [&str; 5] = [
        "Keine Aktion",
        "No action",
        "Aucune action",
        "Ninguna acción",
        "Nessuna azione",
    ];
    pub const DATA_DIRECTORY: [&str; 5] = [
        "Datenverzeichnis",
        "Data directory",
        "Dossier des données",
        "Carpeta de datos",
        "Cartella dei dati",
    ];
    pub const SAVE: [&str; 5] = ["Speichern", "Save", "Enregistrer", "Guardar", "Salva"];
    pub const ADDRESS: [&str; 5] = ["Adresse", "Address", "Adresse", "Dirección", "Indirizzo"];
    pub const LAST_SCAN: [&str; 5] = [
        "letzter Scan",
        "last scan",
        "dernière recherche",
        "última búsqueda",
        "ultima ricerca",
    ];
    pub const CONNECTED: [&str; 5] = [
        "Verbunden",
        "Connected",
        "Connecté",
        "Conectada",
        "Connessa",
    ];
    pub const YES: [&str; 5] = ["Ja", "Yes", "Oui", "Sí", "Sì"];
    pub const NO: [&str; 5] = ["Nein", "No", "Non", "No", "No"];
    pub const CHANNEL: [&str; 5] = ["Kanal", "Channel", "Canal", "Canal", "Canale"];
    pub const ERROR: [&str; 5] = ["Fehler", "Error", "Erreur", "Error", "Errore"];
    pub const DEVICE_DIAGNOSTICS: [&str; 5] = [
        "Gerätediagnose",
        "Device diagnostics",
        "Diagnostic de la station",
        "Diagnóstico de la estación",
        "Diagnostica della stazione",
    ];
    pub const REFRESH: [&str; 5] = [
        "Aktualisieren",
        "Refresh",
        "Actualiser",
        "Actualizar",
        "Aggiorna",
    ];
    pub const COPY_ALL: [&str; 5] = [
        "Alles kopieren",
        "Copy all",
        "Tout copier",
        "Copiar todo",
        "Copia tutto",
    ];
    pub const EXPORT: [&str; 5] = ["Exportieren", "Export", "Exporter", "Exportar", "Esporta"];
    pub const CONFIRM_DEVICE_ACTION: [&str; 5] = [
        "Geräteaktion bestätigen",
        "Confirm device action",
        "Confirmer la commande",
        "Confirmar acción",
        "Conferma comando",
    ];
    pub const STATIONS: [&str; 5] = [
        "Stationen",
        "stations",
        "stations",
        "estaciones",
        "stazioni",
    ];
    pub const EXECUTE_NOW: [&str; 5] = [
        "Jetzt ausführen",
        "Execute now",
        "Exécuter",
        "Ejecutar",
        "Esegui",
    ];
    pub const CANCEL: [&str; 5] = ["Abbrechen", "Cancel", "Annuler", "Cancelar", "Annulla"];
    pub const WORKING: [&str; 5] = [
        "Vorgang läuft…",
        "Working…",
        "Opération en cours…",
        "Procesando…",
        "Operazione in corso…",
    ];
    pub const SCAN: [&str; 5] = ["Suchen", "Scan", "Rechercher", "Buscar", "Cerca"];
    pub const RESET_BLUETOOTH: [&str; 5] = [
        "Bluetooth zurücksetzen",
        "Reset Bluetooth",
        "Réinitialiser Bluetooth",
        "Restablecer Bluetooth",
        "Ripristina Bluetooth",
    ];
    pub const RESETS_ONLY_THIS_APPLICATION_S_CONNECTIONS: [&str; 5] = [
        "Setzt nur die Verbindungen dieser Anwendung zurück",
        "Resets only this application's connections",
        "Réinitialise uniquement les connexions de cette application",
        "Solo restablece las conexiones de esta aplicación",
        "Ripristina solo le connessioni di questa applicazione",
    ];
    pub const BLE_DISABLED_UI_TEST: [&str; 5] = [
        "BLE deaktiviert — Oberflächentest",
        "BLE disabled — UI test",
        "BLE désactivé — test de l’interface",
        "BLE desactivado — prueba de interfaz",
        "BLE disattivato — test dell’interfaccia",
    ];
    pub const INVALID_CONFIGURATION_ORIGINAL_RETAINED_SAVING_DISABLED: [&str; 5] = [
        "Konfiguration fehlerhaft. Original bleibt erhalten; Speichern deaktiviert.",
        "Invalid configuration. Original retained; saving disabled.",
        "Configuration incorrecte. L’original est conservé ; l’enregistrement est désactivé.",
        "Configuración no válida. Se conserva el original y se desactiva el guardado.",
        "Configurazione non valida. Originale conservato; salvataggio disattivato.",
    ];
    pub const SELECT_ALL: [&str; 5] = [
        "Alle auswählen",
        "Select all",
        "Tout sélectionner",
        "Seleccionar todo",
        "Seleziona tutto",
    ];
    pub const CLEAR_SELECTION: [&str; 5] = [
        "Auswahl aufheben",
        "Clear selection",
        "Effacer la sélection",
        "Quitar selección",
        "Deseleziona tutto",
    ];
    pub const SELECTED_LABEL: [&str; 5] = [
        "Ausgewählt",
        "Selected",
        "Sélection",
        "Selección",
        "Selezione",
    ];
    pub const SELECT: [&str; 5] = ["Auswahl", "Select", "Choix", "Selección", "Scelta"];
    pub const NAME: [&str; 5] = ["Name", "Name", "Nom", "Nombre", "Nome"];
    pub const STATE_CONNECTION: [&str; 5] = [
        "Status / Verbindung",
        "State / connection",
        "État / connexion",
        "Estado / conexión",
        "Stato / connessione",
    ];
    pub const CLICK_TO_SORT_CLICK_AGAIN_TO_REVERSE_ORDER: [&str; 5] = [
        "Klicken zum Sortieren; erneut klicken für umgekehrte Reihenfolge",
        "Click to sort; click again to reverse order",
        "Cliquez pour trier ; cliquez à nouveau pour inverser l’ordre",
        "Haz clic para ordenar; repite para invertir el orden",
        "Fai clic per ordinare; ripeti per invertire l’ordine",
    ];
    pub const SELECT_ONLY_THIS_STATION: [&str; 5] = [
        "Nur diese Station auswählen",
        "Select only this station",
        "Sélectionner uniquement cette station",
        "Seleccionar solo esta estación",
        "Seleziona solo questa stazione",
    ];
    pub const CHECKING: [&str; 5] = [
        "Wird abgefragt…",
        "Checking…",
        "Vérification…",
        "Comprobando…",
        "Verifica…",
    ];
    pub const OFFLINE: [&str; 5] = [
        "Offline",
        "Offline",
        "Hors ligne",
        "Sin conexión",
        "Non in linea",
    ];
    pub const UNAVAILABLE: [&str; 5] = [
        "Nicht verfügbar",
        "Unavailable",
        "Indisponible",
        "No disponible",
        "Non disponibile",
    ];
    pub const ON_STATUS: [&str; 5] = ["Ein", "On", "Allumée", "Encendida", "Accesa"];
    pub const UNKNOWN: [&str; 5] = [
        "Unbekannt",
        "Unknown",
        "Inconnu",
        "Desconocido",
        "Sconosciuto",
    ];
    pub const LAST_SCAN_LABEL: [&str; 5] = [
        "Letzter Scan",
        "Last scan",
        "Dernière recherche",
        "Última búsqueda",
        "Ultima ricerca",
    ];
    pub const DRAG_TO_RESIZE_DEVICE_LIST_AND_LOG: [&str; 5] = [
        "Ziehen: Höhe von Geräteliste und Log ändern",
        "Drag to resize device list and log",
        "Faites glisser pour redimensionner la liste et le journal",
        "Arrastra para ajustar la lista y el registro",
        "Trascina per ridimensionare elenco e registro",
    ];
    pub const NO_STATIONS_FOUND_ENABLE_BLUETOOTH_AND_START_A_SCAN: [&str; 5] = [
        "Keine Stationen gefunden. Bluetooth aktivieren und Suche starten.",
        "No stations found. Enable Bluetooth and start a scan.",
        "Aucune station trouvée. Activez Bluetooth et lancez une recherche.",
        "No se encontraron estaciones. Activa Bluetooth e inicia una búsqueda.",
        "Nessuna stazione trovata. Attiva Bluetooth e avvia una ricerca.",
    ];
    pub const LOCAL_NAME: [&str; 5] = [
        "Lokaler Name:",
        "Local name:",
        "Nom local :",
        "Nombre local:",
        "Nome locale:",
    ];
    pub const CHANNEL_LABEL: [&str; 5] = ["Kanal:", "Channel:", "Canal :", "Canal:", "Canale:"];
    pub const CHANGE_CHANNEL: [&str; 5] = [
        "Kanal ändern",
        "Change channel",
        "Changer de canal",
        "Cambiar canal",
        "Cambia canale",
    ];
    pub const DIAGNOSTICS: [&str; 5] = [
        "Diagnose",
        "Diagnostics",
        "Diagnostic",
        "Diagnóstico",
        "Diagnostica",
    ];
    pub const LOG: [&str; 5] = ["Log", "Log", "Journal", "Registro", "Registro"];
    pub const CLEAR: [&str; 5] = ["Leeren", "Clear", "Effacer", "Vaciar", "Svuota"];
    pub const SAVE_EXPORT: [&str; 5] = [
        "Speichern / Export",
        "Save / Export",
        "Enregistrer / Exporter",
        "Guardar / Exportar",
        "Salva / Esporta",
    ];
    pub const SEARCH: [&str; 5] = ["Suchen…", "Search…", "Rechercher…", "Buscar…", "Cerca…"];
    pub const ALL_LEVELS: [&str; 5] = [
        "Alle Level",
        "All levels",
        "Tous les niveaux",
        "Todos los niveles",
        "Tutti i livelli",
    ];
    pub const ALL: [&str; 5] = ["Alle", "All", "Tous", "Todos", "Tutti"];
}
