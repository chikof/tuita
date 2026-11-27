app_name = tuita
loading = Caricamento
close = Chiudi
dismiss = Chiudi
notifications = Notifiche
avatar_more = altri { $count }
char_counter_left =
    { $count ->
        [one] Resta { $count } carattere
       *[other] Restano { $count } caratteri
    }
char_counter_over =
    { $count ->
        [one] { $count } carattere oltre il limite
       *[other] { $count } caratteri oltre il limite
    }
not_found_title = Non c’è niente qui
not_found_body = Questa pagina non esiste o è stata spostata.
not_found_back = Indietro
tagline = Ogni post, due inchiostri.
error_title = Qualcosa è andato storto
cancel = Annulla
retry = Riprova
back = Indietro
view = Visualizza
follow = Segui
following = Seguiti
follows_you = Ti segue
verified = Account verificato
bot_account = Account automatizzato
bot_label = Bot
link_copied = Link copiato negli appunti
text_copied = Testo copiato
time_now = ora
time_minutes = { $n } min
time_hours = { $n } h
time_days = { $n } g
nav_main = Principale
nav_home = Home
nav_explore = Esplora
nav_notifications = Notifiche
nav_bookmarks = Segnalibri
nav_profile = Profilo
nav_settings = Impostazioni
nav_compose = Pubblica
nav_menu = Menu
badge_unread = { $count } non lette
account_menu = Account
account_switch = Cambia account
account_switch_to = Passa a { $handle }
account_add = Aggiungi un account
account_logout = Esci da { $handle }
account_logout_all = Esci da tutti gli account
home_feed = Timeline
home_for_you = Per te
home_following = Seguiti
home_empty_title = Ancora niente qui
home_empty_body = Pubblica tu per primo.
home_following_empty_title = La tua cerchia è silenziosa
home_following_empty_body = Segui qualcuno per riempire questa timeline con i suoi post e repost.
home_find_people = Trova persone
new_posts =
    { $count ->
        [one] Mostra { $count } nuovo post
       *[other] Mostra { $count } nuovi post
    }
new_posts_many = Mostra più di 99 nuovi post
composer_title = Nuovo post
composer_reply_title = Rispondi
composer_quote_title = Cita post
composer_placeholder = Che succede?
composer_reply_placeholder = Pubblica la tua risposta
composer_text_label = Testo del post
composer_replying_to = In risposta a { $handle }
composer_cw_label = Avviso sui contenuti
composer_cw_placeholder = Avviso sui contenuti, ad es. spoiler
composer_cw_toggle = Aggiungi un avviso sui contenuti
composer_sensitive_toggle = Segna i media come sensibili
composer_options = Opzioni del post
composer_policy_label = Chi può rispondere
composer_suggestions = Suggerimenti
composer_tag_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } post
    }
composer_post = Pubblica
composer_reply = Rispondi
policy_everyone = Tutti possono rispondere
policy_following = Possono rispondere le persone che segui
policy_mentioned = Possono rispondere solo le persone che menzioni
post_sent = Post inviato
post_by = Post di { $name }
post_reposted_by = { $name } ha ripubblicato
post_pinned = Post fissato
post_replying_to = In risposta a { $handle }
post_replying_unavailable = In risposta a un post non disponibile
post_unavailable = Questo post non è disponibile.
post_menu = Altro
post_copy_link = Copia link
post_copy_text = Copia testo
post_pin = Fissa sul profilo
post_unpin = Non fissare più sul profilo
post_pinned_toast = Fissato sul tuo profilo
post_unpinned = Non più fissato sul tuo profilo
post_delete = Elimina post
post_delete_title = Eliminare il post?
post_delete_body = Non si può annullare. Verrà rimosso dal tuo profilo, dalle timeline di chi ti segue e dalla ricerca.
post_deleted = Post eliminato
post_view_profile = Vedi { $handle }
post_mute = Silenzia { $handle }
post_block = Blocca { $handle }
post_reply_label = Rispondi ({ $count })
post_reply_following_only = Solo le persone seguite dall’autore possono rispondere.
post_reply_mentioned_only = Solo le persone menzionate in questo post possono rispondere.
post_repost_menu = Opzioni di repost
post_repost = Ripubblica
post_undo_repost = Annulla repost
post_quote = Cita
post_reposts_label = Repost e citazioni ({ $count })
post_like_label = Mi piace ({ $count })
post_bookmark_label = Aggiungi ai segnalibri
post_share = Condividi o copia il link
cw_show = Mostra
cw_hide = Nascondi
cw_label = Avviso sui contenuti: { $reason }
muted_toast = { $handle } silenziato
unmuted_toast = { $handle } non è più silenziato
blocked_toast = { $handle } bloccato
unblocked_toast = { $handle } sbloccato
block_title = Bloccare { $handle }?
block_body = Non potrà seguirti e tu non vedrai i suoi post. Non riceverà alcun avviso.
block_confirm = Blocca
notify_like = @{ $handle } ha messo mi piace al tuo post
notify_repost = @{ $handle } ha ripubblicato il tuo post
notify_reply = @{ $handle } ti ha risposto
notify_mention = @{ $handle } ti ha menzionato
notify_quote = @{ $handle } ha citato il tuo post
notify_follow = @{ $handle } ha iniziato a seguirti
notify_follow_request = @{ $handle } ha chiesto di seguirti
notify_poll = Un sondaggio a cui hai partecipato è terminato
notify_affiliation = @{ $handle } ti ha invitato ad affiliarti
shortcuts_title = Scorciatoie da tastiera
shortcut_group_navigation = Navigazione
shortcut_group_actions = Azioni
shortcut_or = /
shortcut_then = poi
shortcut_plus = +
shortcut_next_prev = Post successivo / precedente
shortcut_open = Apri post
shortcut_search = Cerca
shortcut_go_home = Home
shortcut_go_explore = Esplora
shortcut_go_notifications = Notifiche
shortcut_go_bookmarks = Segnalibri
shortcut_go_profile = Il tuo profilo
shortcut_go_settings = Impostazioni
shortcut_new_post = Nuovo post
shortcut_like = Mi piace
shortcut_reply = Rispondi
shortcut_repost = Ripubblica o cita
shortcut_bookmark = Aggiungi ai segnalibri
shortcut_send = Invia il post
shortcut_help = Mostra questa guida
auth_login_title = Accedi
auth_add_title = Aggiungi un altro account
auth_add_sub = Accedi per passare da un account all’altro, fino a { $max } su questo dispositivo, senza uscire.
auth_signup_title = Crea account
auth_handle = Nome utente
auth_password = Password
auth_display_name = Nome visualizzato
auth_handle_hint = 3–15 lettere, numeri o trattini bassi
auth_password_hint = Almeno 8 caratteri
auth_consent_pre = Ho almeno { $minAge } anni e accetto i{" "}
auth_consent_terms = Termini di servizio
auth_consent_and = {" "}e l’
auth_consent_privacy = Informativa sulla privacy
auth_consent_end = {"."}
auth_consent_required = Accetta i Termini di servizio e l’Informativa sulla privacy.
auth_signin = Accedi
auth_signin_add = Accedi e passa
auth_create = Crea account
auth_to_signup = Sei nuovo? Crea un account
auth_to_login = Hai un account? Accedi
auth_closed = Al momento le iscrizioni a questa istanza sono chiuse.
landing_nav = Piè di pagina
landing_rules = Regole dell’istanza
landing_stats = In questo momento
landing_people = Persone
landing_posts = Post
landing_clock = La tua ora locale
landing_trends = Di tendenza
landing_trend =
    { $count ->
        [one] { $count } post
       *[other] { $count } post
    }
landing_join = Unisciti a { $name }
landing_join_sub = Crea un account e pubblica in due inchiostri.
landing_closed = Le iscrizioni sono chiuse al momento. Hai già un account? Accedi.
landing_terms = Termini
landing_privacy = Privacy
landing_source = Codice sorgente
landing_powered = Basato su tuita { $version }
docs_terms_title = Termini di servizio
docs_privacy_title = Informativa sulla privacy
docs_updated = Ultimo aggiornamento: { $date }
docs_date = 5 novembre 2026
docs_see_privacy = Vedi anche l’Informativa sulla privacy.
docs_see_terms = Vedi anche i Termini di servizio.
docs_contact_fallback = gli amministratori dell’istanza (tramite messaggio diretto o segnalazione)
docs_terms_agreement_h = 1. Accordo
docs_terms_agreement_p = I presenti Termini di servizio («Termini») regolano l’uso di { $name }, l’istanza tuita all’indirizzo { $domain } («il servizio», «noi»). Creando un account o usando il servizio accetti questi Termini e la nostra Informativa sulla privacy. Se non sei d’accordo, non usare il servizio.
docs_terms_eligibility_h = 2. Chi può usarlo
docs_terms_eligibility_p = Devi avere almeno { $minAge } anni, o l’età minima per il consenso digitale nel tuo Paese se è più alta. Devi poter stipulare un accordo vincolante e non essere escluso dall’uso del servizio dalla legge applicabile.
docs_terms_account_h = 3. Il tuo account
docs_terms_account_1 = Mantieni segreta la tua password. Sei responsabile dell’attività del tuo account.
docs_terms_account_2 = Il tuo nome utente non deve imitare un’altra persona o organizzazione né ingannare gli altri su chi sei.
docs_terms_account_3 = Avvisaci subito se pensi che il tuo account sia stato compromesso. Puoi chiudere le altre sessioni da Impostazioni → Account.
docs_terms_content_h = 4. I tuoi contenuti
docs_terms_content_p1 = Resti proprietario di ciò che pubblichi. Ci concedi una licenza mondiale, non esclusiva e gratuita per archiviare, mostrare, riprodurre e distribuire i tuoi contenuti solo per far funzionare e migliorare il servizio. Questo include l’invio alle istanze tuita federate (vedi la sezione 6). La licenza termina quando elimini il contenuto o l’account, tranne che per le copie già consegnate ad altre istanze o salvate da altre persone.
docs_terms_content_p2 = Sei responsabile di ciò che pubblichi. Pubblica solo contenuti che hai il diritto di condividere.
docs_terms_rules_h = 5. Regole della comunità
docs_terms_rules_lead = Usando { $name } accetti di non violare la legge né queste regole:
docs_terms_rules_p = Accetti inoltre di non attaccare il servizio, ad esempio aggirando le misure di sicurezza, estraendone dati su larga scala, inviando malware o sovraccaricandolo.
docs_terms_federation_h = 6. Federazione
docs_terms_federation_p = { $name } può connettersi («federarsi») con altre istanze tuita indipendenti approvate dagli amministratori. Quando interagisci con persone di un’altra istanza, il tuo profilo pubblico, i post pubblici e l’interazione (ad esempio un follow, un mi piace, un repost, una risposta o un messaggio diretto) vengono inviati a quell’istanza. Le altre istanze sono gestite da altre persone con i propri termini. Non possiamo controllare né eliminare i contenuti una volta che un’altra istanza li ha ricevuti, ma inviamo richieste di eliminazione quando cancelli qualcosa.
docs_terms_moderation_h = 7. Moderazione e applicazione
docs_terms_moderation_p = I moderatori possono rimuovere contenuti, limitare funzioni o sospendere account che violano questi Termini o la legge. Quando possibile ti spiegheremo il motivo. Chiunque può segnalare un post o un account. Le segnalazioni arrivano ai moderatori di questa istanza. Possiamo smettere di federarci con qualsiasi istanza in qualsiasi momento.
docs_terms_termination_h = 8. Uscire dal servizio
docs_terms_termination_p = Puoi esportare i tuoi dati o eliminare l’account in qualsiasi momento da Impostazioni → Account. Possiamo sospendere o terminare il tuo accesso se violi questi Termini in modo grave o ripetuto. Possiamo anche chiudere il servizio, cercando di darti un preavviso ragionevole.
docs_terms_disclaimers_h = 9. Esclusioni e responsabilità
docs_terms_disclaimers_p = Il servizio è fornito «così com’è» e «come disponibile», senza garanzie di alcun tipo, nella misura massima consentita dalla legge. Non siamo responsabili di danni indiretti, incidentali o consequenziali, né dei contenuti pubblicati dagli utenti o da altre istanze. Nulla in questi Termini limita responsabilità che per legge non possono essere limitate, né i tuoi diritti di consumatore.
docs_terms_changes_h = 10. Modifiche
docs_terms_changes_p = Possiamo aggiornare questi Termini. Se la modifica è rilevante, lo comunicheremo nell’app prima che entri in vigore. Continuare a usare il servizio dopo tale data significa accettare i nuovi Termini.
docs_terms_contact_h = 11. Contatti
docs_terms_contact_lead = Domande su questi Termini:
docs_privacy_summary_h = In breve
docs_privacy_summary_1 = Nessuna pubblicità, nessuno script di tracciamento, nessuna vendita di dati.
docs_privacy_summary_2 = Un solo cookie, per mantenerti connesso. Le preferenze del tema restano nel tuo browser.
docs_privacy_summary_3 = I post pubblici sono pubblici e possono raggiungere le istanze tuita federate.
docs_privacy_summary_4 = I messaggi diretti tra account di questo server sono cifrati end-to-end quando entrambe le persone li attivano: vengono conservati come testo che il server non può leggere, quindi una fuga del database o uno sguardo ai dati conservati non rivela nulla. I messaggi alle persone di altri server e quelli inviati prima di attivare la cifratura non sono cifrati in questo modo e gli amministratori possono tecnicamente leggerli. Questo server fornisce anche l’app che cifra, quindi un amministratore che la modificasse potrebbe catturare ciò che scrivi.
docs_privacy_summary_5 = Puoi esportare o eliminare i tuoi dati quando vuoi.
docs_privacy_controller_h = 1. Chi siamo
docs_privacy_controller_p = Questa informativa riguarda { $name } all’indirizzo { $domain }, gestita dagli amministratori dell’istanza, che decidono come vengono trattati i tuoi dati.
docs_privacy_collect_h = 2. Cosa raccogliamo
docs_privacy_collect_1 = Dati dell’account: nome utente, nome visualizzato, password (conservata solo come hash argon2id con salt) e i dettagli del profilo: bio, località, sito web, avatar, banner, forma e colore.
docs_privacy_collect_2 = Contenuti: post, immagini, risposte, mi piace, repost, segnalibri, follow, blocchi, silenziamenti e segnalazioni.
docs_privacy_collect_3 = Messaggi diretti: conservati su questo server per poter essere consegnati a te e all’altra persona. Un messaggio cifrato end-to-end è conservato come testo cifrato che non possiamo leggere; chi lo ha inviato, a chi e quando restano visibili a noi. Gli altri messaggi sono conservati in chiaro. La tua chiave di cifratura è conservata qui solo in una forma bloccata dalla tua frase segreta, che non vediamo mai.
docs_privacy_collect_4 = Dati di sessione: per ogni dispositivo connesso, lo user-agent del browser, l’indirizzo IP e l’ultimo utilizzo, così puoi controllare e chiudere le sessioni.
docs_privacy_collect_5 = Dati di sicurezza di breve durata: indirizzi IP tenuti in memoria per pochi minuti per limitare le richieste.
docs_privacy_collect_p = Non raccogliamo email, numero di telefono, contatti o posizione, a meno che tu non li inserisca nel tuo profilo.
docs_privacy_use_h = 3. Come li usiamo
docs_privacy_use_p = Solo per far funzionare il servizio: mostrare profilo e post, creare timeline e notifiche, consegnare messaggi, proteggere gli account, prevenire gli abusi e applicare i Termini. La timeline «Per te» ordina i post in base a mi piace, repost, risposte ed età. Non ti profila.
docs_privacy_visibility_h = 4. Cosa possono vedere gli altri
docs_privacy_visibility_p = Il tuo profilo, i post, le risposte, i repost, i mi piace, i follower e i seguiti sono pubblici per chiunque sia connesso a questa istanza e per le istanze federate. Quando qualcuno condivide il link a un post o a un profilo, app come Discord, Slack o iMessage possono recuperare un’anteprima con il post pubblico, le sue immagini e i dettagli pubblici del profilo. I post con avviso sui contenuti o segnati come sensibili hanno un’anteprima senza testo e immagini. Segnalibri, blocchi, silenziamenti, segnalazioni e impostazioni sono privati. Se disattivi «Mostrami nei suggerimenti», le persone possono comunque trovarti con il tuo nome utente esatto.
docs_privacy_federation_h = 5. Federazione
docs_privacy_federation_p = Quando segui, rispondi, menzioni, metti mi piace, ripubblichi o scrivi a qualcuno su un’altra istanza tuita, inviamo a quell’istanza i dati necessari: il tuo profilo pubblico, il post o l’interazione e, per i messaggi diretti, il testo del messaggio. I messaggi diretti verso altre istanze non sono cifrati end-to-end. Le istanze dei tuoi follower ricevono i post pubblici e gli aggiornamenti del profilo. Queste istanze sono indipendenti e conservano i dati secondo le proprie politiche. Quando elimini un post o l’account chiediamo alle istanze federate di eliminare le copie, ma non possiamo garantire che lo facciano.
docs_privacy_cookies_h = 6. Cookie e archiviazione locale
docs_privacy_cookies_p = Usiamo un solo cookie di sessione HttpOnly, strettamente necessario. Le preferenze di tema, colore, dimensione del testo e movimento sono salvate nell’archiviazione locale del browser e non ci vengono mai inviate.
docs_privacy_retention_h = 7. Per quanto tempo conserviamo i dati
docs_privacy_retention_1 = I contenuti e i dati dell’account sono conservati finché non li elimini.
docs_privacy_retention_2 = I post eliminati scompaiono subito dalla vista pubblica e il loro testo e le immagini vengono cancellati.
docs_privacy_retention_3 = Quando elimini l’account rimuoviamo post, mi piace, follow, messaggi e sessioni, e rendiamo anonimo il record dell’account.
docs_privacy_retention_4 = Le sessioni scadono dopo 30 giorni, o subito quando esci.
docs_privacy_retention_5 = Le segnalazioni di moderazione e il registro di controllo dell’amministrazione sono conservati per il tempo necessario a mantenere il servizio sicuro.
docs_privacy_rights_h = 8. I tuoi diritti
docs_privacy_rights_p = A seconda di dove vivi (ad esempio ai sensi del GDPR o del CCPA) puoi avere il diritto di accedere ai tuoi dati, correggerli, esportarli o eliminarli, e di opporti al trattamento o limitarlo. Quasi tutto puoi farlo da solo in Impostazioni → Account: modificare il profilo, scaricare i tuoi dati ed eliminare l’account. Per tutto il resto, contatta:
docs_privacy_rights_end = Puoi anche presentare reclamo all’autorità locale per la protezione dei dati.
docs_privacy_security_h = 9. Sicurezza
docs_privacy_security_p = Le password sono protette con hash scrypt. Dei token di sessione conserviamo solo gli hash. Le richieste tra istanze federate sono firmate con chiavi Ed25519. Nessun sistema è perfettamente sicuro, quindi usa una password unica.
docs_privacy_children_h = 10. Minori
docs_privacy_children_p = Il servizio non è destinato ai minori sotto l’età minima indicata nei Termini. Se pensi che un account appartenga a un minore, segnalalo.
docs_privacy_changes_h = 11. Modifiche
docs_privacy_changes_p = Pubblicheremo qui le modifiche all’informativa e aggiorneremo la data in alto. Le modifiche rilevanti saranno annunciate nell’app.
docs_privacy_contact_h = 12. Contatti
docs_privacy_contact_lead = Domande o richieste sulla privacy:
profile_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } post
    }
profile_edit = Modifica profilo
profile_more = Altro
profile_copy_link = Copia link del profilo
profile_mute = Silenzia { $handle }
profile_unmute = Riattiva { $handle }
profile_block = Blocca { $handle }
profile_unblock = Sblocca { $handle }
profile_unblock_button = Sblocca
profile_suspended = Questo account è sospeso. Solo lo staff può vederlo.
profile_muted = Silenziato
profile_org = Organizzazione
profile_role_admin = Amministratore
profile_role_moderator = Moderatore
profile_joined = Iscritto da { $date }
profile_following_label = Seguiti
profile_followers =
    { $count ->
        [one] Follower
       *[other] Follower
    }
profile_counts = Numero di follower e seguiti
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Seguito da { $a }
            }
        [2]
            { $others ->
               *[other] Seguito da { $a } e { $b }
            }
       *[other]
            { $others ->
                [one] Seguito da { $a }, { $b } e { $others } altro che segui
               *[other] Seguito da { $a }, { $b } e altri { $others } che segui
            }
    }
profile_banner = Banner del profilo
profile_tabs = Sezioni del profilo
profile_tab_posts = Post
profile_tab_replies = Risposte
profile_tab_media = Media
profile_tab_likes = Mi piace
profile_posts_label = Post
profile_empty_posts_self_title = Non hai ancora pubblicato
profile_empty_posts_self_body = Qui compariranno i tuoi post e repost.
profile_empty_posts_title = { $handle } non ha ancora pubblicato
profile_empty_posts_body = Quando lo farà, i suoi post compariranno qui.
profile_empty_replies_title = Ancora nessuna risposta
profile_empty_replies_body = Le risposte compariranno qui.
profile_empty_media_title = Ancora nessuna foto
profile_empty_media_body = Qui compariranno i post con immagini.
profile_empty_likes_title = Ancora nessun mi piace
profile_empty_likes_body = Qui compariranno i post che ti piacciono.
profile_blocked_title = Hai bloccato { $handle }
profile_blocked_body = Non vedrai i suoi post e non potrà seguirti né scriverti.
profile_hidden_title = Non puoi vedere questi post
profile_hidden_body = { $handle } ha limitato chi può vedere il suo profilo.
follows_tabs = Elenchi di follower e seguiti
follows_tab_followers = Follower
follows_tab_following = Seguiti
follows_label_followers = Follower
follows_label_following = Seguiti
follows_empty_followers_title = Ancora nessun follower
follows_empty_followers_body = Quando qualcuno seguirà questo account, comparirà qui.
follows_empty_following_title = Non segue nessuno
follows_empty_following_body = Gli account che segue compariranno qui.
thread_title = Post
thread_doc_title = { $name }: «{ $text }»
thread_views =
    { $count ->
        [one] Visualizzazione
       *[other] Visualizzazioni
    }
thread_reposts =
    { $count ->
        [one] Repost
       *[other] Repost
    }
thread_quotes =
    { $count ->
        [one] Citazione
       *[other] Citazioni
    }
thread_likes =
    { $count ->
        [one] Mi piace
       *[other] Mi piace
    }
thread_stats = Statistiche del post
thread_replies = Risposte
thread_hidden = Risposte nascoste ({ $count })
thread_hidden_note = L’autore ha nascosto queste risposte.
thread_list_likes = Piace a
thread_list_reposts = Ripubblicato da
thread_list_quotes = Citazioni
thread_list_empty = Ancora nessuno
tag_empty_title = Nessun post con { $tag }
tag_empty_body = Sii il primo a usarlo.
tag_posts_label = Post con { $tag }
bookmarks_title = Segnalibri
bookmarks_label = Post nei segnalibri
bookmarks_empty_title = Salva i post per dopo
bookmarks_empty_body = Tocca l’icona segnalibro su un post per tenerlo qui. Solo tu vedi i tuoi segnalibri.
settings_account = Il tuo account
settings_account_desc = Profilo e password
settings_profile = Modifica profilo
settings_profile_desc = Nome, bio, forma dell’avatar e inchiostro
settings_privacy = Privacy e sicurezza
settings_privacy_desc = Messaggi, visibilità, parole silenziate, account bloccati e silenziati
settings_notifications = Notifiche
settings_notifications_desc = Scegli di cosa ricevere notifiche
settings_appearance = Aspetto
settings_appearance_desc = Tema, inchiostro, dimensione del testo, contrasto e movimento
settings_language = Lingua
settings_language_desc = Lingua dell’interfaccia
settings_language_note = Salvata in questo browser e nel tuo account quando sei connesso.
settings_about = Informazioni su questa istanza
settings_about_desc = Regole, termini, privacy, versione e scorciatoie da tastiera
settings_logout = Esci
settings_account_profile = Profilo
settings_account_joined = Iscritto da { $date }
settings_account_edit = Modifica profilo
settings_password_title = Cambia password
settings_password_current = Password attuale
settings_password_new = Nuova password
settings_password_hint = Almeno 8 caratteri. Gli altri dispositivi verranno disconnessi.
settings_password_confirm = Conferma la nuova password
settings_password_submit = Aggiorna password
settings_password_mismatch = Le nuove password non coincidono.
settings_password_updated = Password aggiornata. Le altre sessioni sono state chiuse.
settings_sessions_title = Dispositivi e sessioni
settings_sessions_desc = Esci da tutto ciò che non riconosci, poi cambia la password.
settings_device_on = { $browser } su { $os }
settings_device_unknown = Dispositivo sconosciuto
settings_session_current = Questo dispositivo
settings_session_last_active = Ultima attività: { $time }
settings_session_sign_out = Esci
settings_session_sign_out_device = Esci da { $device }
settings_session_signed_out = Sessione chiusa
settings_sessions_revoke_all = Esci da tutti gli altri dispositivi
settings_sessions_revoke_title = Uscire dagli altri dispositivi?
settings_sessions_revoke_body = Tutti i dispositivi tranne questo verranno disconnessi.
settings_sessions_revoke_confirm = Disconnetti gli altri
settings_sessions_revoked =
    { $count ->
        [one] { $count } sessione chiusa
       *[other] { $count } sessioni chiuse
    }
settings_profile_preview = Anteprima
settings_profile_preview_alt = Anteprima dell’avatar
settings_profile_uploads = Il caricamento di avatar e banner arriverà più avanti. Per ora vengono usati l’avatar e il banner generati.
settings_profile_details = Dettagli
settings_profile_name = Nome visualizzato
settings_profile_bio = Bio
settings_profile_location = Località
settings_profile_website = Sito web
settings_profile_shape = Forma dell’avatar
settings_profile_ink = Inchiostro del profilo
settings_profile_save = Salva profilo
settings_profile_saved = Profilo aggiornato
settings_shape_square = Quadrato
settings_shape_circle = Cerchio
settings_shape_squircle = Arrotondato
settings_shape_octagon = Ottagono
settings_ink_bone = Avorio
settings_ink_phosphor = Fosforo
settings_ink_amber = Ambra
settings_ink_cyan = Ciano
settings_ink_rose = Rosa
settings_dm_title = Messaggi diretti
settings_dm_desc = Chi può iniziare una conversazione con te
settings_dm_label = Chi può scriverti
settings_dm_everyone = Tutti
settings_dm_following = Persone che seguo
settings_dm_none = Nessuno
settings_dm_saved = Impostazioni dei messaggi salvate
settings_discover_title = Visibilità
settings_discoverable = Mostrami nei suggerimenti e nella ricerca
settings_discoverable_hint = Le persone possono comunque trovarti con il tuo nome utente esatto
settings_show_client = Mostra da quale dispositivo pubblico
settings_show_client_hint = Aggiunge il dispositivo sotto i tuoi nuovi post
settings_words_title = Parole silenziate
settings_words_desc = Nascondi da timeline, ricerca e risposte i post che contengono queste parole, frasi o #hashtag. I tuoi post non vengono mai nascosti.
settings_words_label = Parola da silenziare
settings_words_placeholder = Aggiungi una parola, frase o #hashtag
settings_words_add = Silenzia
settings_words_remove = Riattiva { $word }
settings_words_none = Nessuna parola silenziata.
settings_words_limit = Puoi silenziare fino a { $count } parole.
settings_blocked_title = Account bloccati
settings_blocked_desc = Non possono seguirti, scriverti né vedere o interagire con i tuoi post. Non ricevono alcun avviso.
settings_blocked_empty = Non hai bloccato nessuno.
settings_muted_title = Account silenziati
settings_muted_desc = Non vedrai i suoi post né le sue notifiche. Potrà comunque seguirti e scriverti.
settings_muted_empty = Non hai silenziato nessuno.
settings_unblock = Sblocca
settings_unmute = Riattiva
settings_unblock_user = Sblocca { $handle }
settings_unmute_user = Riattiva { $handle }
settings_notif_title = Avvisami per
settings_notif_desc = Gli account silenziati e bloccati non ti inviano mai notifiche.
settings_notif_like = Mi piace
settings_notif_like_hint = Quando qualcuno mette mi piace a un tuo post
settings_notif_repost = Repost
settings_notif_repost_hint = Quando qualcuno ripubblica un tuo post
settings_notif_reply = Risposte
settings_notif_reply_hint = Quando qualcuno ti risponde
settings_notif_mention = Menzioni
settings_notif_mention_hint = Quando qualcuno ti menziona
settings_notif_quote = Citazioni
settings_notif_quote_hint = Quando qualcuno cita un tuo post
settings_notif_follow = Nuovi follower
settings_notif_follow_hint = Quando qualcuno ti segue
settings_desktop_title = Notifiche desktop
settings_desktop_desc = Mostrate come notifiche di sistema solo quando questa scheda è in secondo piano.
settings_desktop_state = Autorizzazione: { $state }
settings_desktop_allowed = Consentita
settings_desktop_blocked = Bloccata
settings_desktop_not_asked = Non richiesta
settings_desktop_enable = Attiva
settings_desktop_blocked_toast = Le notifiche per questo sito sono bloccate nelle impostazioni del browser.
settings_desktop_unsupported = Questo browser non supporta le notifiche desktop.
settings_theme_title = Tema
settings_appearance_note = Salvato su questo dispositivo e applicato subito.
settings_theme_mode = Tema
settings_theme_system = Sistema
settings_theme_light = Chiaro
settings_theme_dark = Scuro
settings_theme_ink = Inchiostro
settings_theme_contrast = Contrasto
settings_contrast_normal = Normale
settings_contrast_more = Maggiore
settings_reading_title = Testo e movimento
settings_theme_text_size = Dimensione del testo
settings_theme_motion = Movimento
settings_motion_system = Sistema
settings_motion_reduce = Ridotto
settings_preview_title = Anteprima
settings_preview_text = Così appariranno i post con le tue impostazioni.
settings_about_domain = Dominio
settings_about_version = Versione
settings_about_contact = Contatti
settings_about_terms = Termini di servizio
settings_about_privacy = Informativa sulla privacy
settings_about_source = Codice sorgente
settings_about_rules = Regole
explore_search_label = Cerca post, persone e tag
explore_search_placeholder = Cerca post, persone, #tag
explore_clear = Cancella ricerca
explore_tips = Suggerimenti di ricerca
explore_tips_title = Operatori di ricerca
explore_op_from = Post di un account
explore_op_to = Risposte o menzioni a un account
explore_op_media = Post con immagini (o has:poll, has:links)
explore_op_since = Pubblicati a partire da una data
explore_op_until = Pubblicati fino a una data
explore_op_phrase = Parole in questo ordine
explore_op_exclude = Escludi i post con una parola
explore_results = Risultati della ricerca
explore_tab_posts = Post
explore_tab_people = Persone
explore_tab_media = Media
explore_people = Persone
explore_posts = Post
explore_recent = Ricerche recenti
explore_recent_clear = Cancella
explore_trending = Di tendenza
explore_trend_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } post
    }
explore_trends_empty = Ancora nessuna tendenza. Usa un #hashtag!
explore_suggestions = Chi seguire
explore_suggestions_empty = Segui tutti. Notevole.
explore_no_posts_title = Nessun post per «{ $q }»
explore_no_posts_body = Prova con un’altra parola o hashtag.
explore_no_people_title = Nessuna persona per «{ $q }»
explore_no_people_body = Prova un nome, un nome utente, oppure @utente@istanza per qualcuno su un’altra istanza.
explore_no_media_title = Nessuna immagine per «{ $q }»
explore_no_media_body = Qui compariranno i post con immagini corrispondenti.
notif_filter = Filtro notifiche
notif_tab_all = Tutte
notif_tab_mentions = Menzioni
notif_empty_all_title = Ancora nessuna notifica
notif_empty_all_body = Qui compariranno mi piace, risposte, repost e nuovi follower.
notif_empty_mentions_title = Ancora nessuna menzione
notif_empty_mentions_body = Qui compaiono risposte, menzioni e citazioni dei tuoi post.
notif_who_two = { $first } e { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } e { $count } altro
       *[other] { $first }, { $second } e altri { $count }
    }
notif_verb_like =
    { $count ->
        [one] ha messo mi piace al tuo post
       *[other] hanno messo mi piace al tuo post
    }
notif_verb_repost =
    { $count ->
        [one] ha ripubblicato il tuo post
       *[other] hanno ripubblicato il tuo post
    }
notif_verb_follow =
    { $count ->
        [one] ha iniziato a seguirti
       *[other] hanno iniziato a seguirti
    }
notif_verb_follow_request =
    { $count ->
        [one] ha chiesto di seguirti
       *[other] hanno chiesto di seguirti
    }
notif_verb_reply =
    { $count ->
        [one] ti ha risposto
       *[other] ti hanno risposto
    }
notif_verb_mention =
    { $count ->
        [one] ti ha menzionato
       *[other] ti hanno menzionato
    }
notif_verb_quote =
    { $count ->
        [one] ha citato il tuo post
       *[other] hanno citato il tuo post
    }
notif_verb_poll =
    { $count ->
        [one] ha chiuso un sondaggio a cui hai partecipato
       *[other] hanno chiuso un sondaggio a cui hai partecipato
    }
notif_verb_affiliation =
    { $count ->
        [one] ti ha invitato ad affiliarti
       *[other] ti hanno invitato ad affiliarti
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Programma
composer_post_all = Pubblica tutto
composer_err_type = Questo file non è un’immagine o un video utilizzabile da tuita (PNG, JPEG, GIF, WebP, MP4, WebM o MOV).
composer_err_image_size = L’immagine è troppo grande (max 10 MB).
composer_err_video_size = I video possono pesare fino a { $mb } MB.
composer_err_max_images = Puoi allegare fino a 4 immagini.
composer_err_exclusive = Un post può avere un video, delle immagini o un sondaggio, ma solo uno di questi.
composer_extra_placeholder = Aggiungi un altro post
composer_extra_label = Post { $n } del thread
composer_extra_remove = Rimuovi il post { $n } del thread
composer_schedule_chip = Verrà pubblicato il { $time }
composer_schedule_clear = Non programmare
composer_add_images = Aggiungi immagini
composer_add_video = Aggiungi un video
composer_add_poll = Aggiungi un sondaggio
composer_add_post = Aggiungi un altro post
composer_scheduled = Programmato per il { $time }
composer_thread_sent = Thread di { $count } post inviato
composer_remove_video = Rimuovi video
composer_remove_image = Rimuovi immagine
composer_uploading = Caricamento in corso
composer_uploading_video = Caricamento del video… { $percent }%
composer_alt_video = Descrivi questo video
composer_alt_image = Descrivi questa immagine
composer_alt_label = Descrizione (testo alternativo) dell’allegato { $index }
composer_emoji = Aggiungi emoji
poll_length_hours = { $n } h
poll_length_days = { $n } g
poll_choice = Opzione { $n }
poll_choice_optional = Opzione { $n } (facoltativa)
poll_remove_choice = Rimuovi l’opzione { $n }
poll_add_choice = Aggiungi un’opzione
poll_length = Durata del sondaggio
poll_multiple_label = Consenti più scelte
poll_remove = Rimuovi sondaggio
schedule_title = Programma post
schedule_hour = Tra un’ora
schedule_evening = Stasera
schedule_tomorrow = Domani mattina
schedule_week = Tra una settimana
schedule_window = Scegli un orario tra un minuto e 90 giorni da adesso.
schedule_custom = Scegli data e ora
schedule_set = Imposta
schedule_clear = Non programmare
schedule_view = Vedi i post programmati
dropzone_drop = Rilascia per allegare
dropzone_reject = Impossibile usare questo file
nav_messages = Messaggi
nav_lists = Liste
nav_scheduled = Programmati
notify_message = @{ $handle } ti ha inviato un messaggio
shortcut_go_messages = Messaggi
shortcut_go_lists = Liste
cropper_stage = Area di ritaglio. Trascina per spostare l’immagine. Usa le frecce per muoverla e più e meno per ingrandire.
cropper_hint = Trascina per riposizionare. Pizzica, scorri o usa il cursore per lo zoom.
cropper_zoom = Zoom
cropper_zoom_in = Ingrandisci
cropper_zoom_out = Rimpicciolisci
cropper_rotate = Ruota
cropper_reset = Reimposta
cropper_apply = Applica
image_field_avatar = Immagine del profilo
image_field_banner = Banner
image_field_edit_avatar = Modifica immagine del profilo
image_field_edit_banner = Modifica banner
image_field_change = Cambia
image_field_remove = Rimuovi
image_field_hint = PNG, JPEG, GIF o WebP fino a 10 MB. Una GIF diventa un’immagine fissa.
image_field_not_image = Questo file non è un’immagine utilizzabile da tuita (PNG, JPEG, GIF o WebP).
image_field_too_large = L’immagine è troppo grande (max 10 MB).
image_field_uploading = Caricamento in corso
image_field_saved = Immagine salvata.
image_field_removed = Immagine rimossa.
edit_title = Modifica post
edit_note = Puoi modificare per 30 minuti dopo la pubblicazione. Gli altri possono vedere che un post è stato modificato e le versioni precedenti.
edit_save = Salva
edit_saved = Post aggiornato
edit_edited = Modificato
edit_history_title = Cronologia modifiche
edit_history_current = Versione attuale
edit_history_earlier = Versione precedente
edit_history_original = Originale
report_post_title = Segnala post
report_user_title = Segnala { $handle }
report_intro = Cosa non va? I moderatori di questa istanza esamineranno la tua segnalazione. { $handle } non vedrà chi l’ha segnalato.
report_reason = Motivo
report_note = Altro? (facoltativo)
report_send = Invia segnalazione
report_sent = Grazie. La segnalazione è stata inviata ai moderatori.
report_duplicate = Hai già inviato questa segnalazione. I moderatori la esamineranno.
report_spam = Spam
report_spam_body = Pubblicità indesiderata, truffe, contenuti ripetitivi o fuorvianti
report_harassment = Molestie
report_harassment_body = Prendere di mira, bullizzare o minacciare qualcuno
report_hate = Odio
report_hate_body = Attacca le persone per ciò che sono
report_violence = Violenza o autolesionismo
report_violence_body = Minacce, esaltazione della violenza, istigazione all’autolesionismo
report_misinformation = Disinformazione
report_misinformation_body = Affermazioni false che potrebbero causare danni
report_impersonation = Furto d’identità
report_impersonation_body = Si spaccia per qualcun altro
report_other = Altro
report_other_body = Viola le regole in altro modo
post_edit = Modifica post
post_edit_history = Cronologia modifiche
post_report = Segnala post
post_hide_reply = Nascondi risposta
post_unhide_reply = Mostra risposta
post_share_menu = Condividi…
post_reply_hidden = Risposta nascosta
post_reply_hidden_toast = Risposta nascosta
post_reply_unhidden_toast = Risposta di nuovo visibile
sensitive_label = Media sensibili
sensitive_show = Mostra
sensitive_hide = Nascondi i media sensibili
media_label =
    { $count ->
        [one] { $count } allegato
       *[other] { $count } allegati
    }
media_video = Video
media_open = Apri l’immagine { $index } di { $count }
lightbox_title = Visualizzatore di media
lightbox_prev = Precedente
lightbox_next = Successivo
poll_vote = Vota
poll_your_vote = Il tuo voto
poll_votes =
    { $count ->
        [one] { $compact } voto
       *[other] { $compact } voti
    }
poll_final = Risultati finali
poll_multiple = scelta multipla
poll_left_minutes = { $n } min rimasti
poll_left_hours = { $n } h rimaste
poll_left_days = { $n } g rimasti
pull_refresh = Trascina per aggiornare
messages_title = Messaggi
messages_new = Nuovo messaggio
messages_label = Conversazioni
messages_empty_title = Inizia una conversazione
messages_empty_body = I messaggi che invii e ricevi compaiono qui.
messages_you_prefix = Tu: { $text }
messages_unread = { $count } non letti
messages_new_search = Cerca persone
messages_new_results = Persone
messages_new_none = Nessuno trovato.
chat_load_older = Carica messaggi precedenti
chat_intro = Questo è l’inizio della conversazione.
chat_label = Messaggi con { $name }
chat_delete = Elimina
chat_seen = Visto
chat_delete_title = Eliminare questo messaggio?
chat_delete_body = Scompare per entrambi.
chat_doc_title = Chat con { $name }
chat_error_title = Impossibile aprire questa conversazione
chat_placeholder = Scrivi un messaggio
chat_input_label = Messaggio
chat_send = Invia
lists_title = Liste
lists_new = Nuova lista
lists_edit = Modifica lista
lists_label = Le tue liste
lists_empty_title = Ancora nessuna lista
lists_empty_body = Raggruppa le persone in liste per leggere i loro post separatamente.
lists_members =
    { $count ->
        [one] { $compact } membro
       *[other] { $compact } membri
    }
lists_private = Privata
lists_name = Nome
lists_description = Descrizione
lists_private_switch = Lista privata
lists_save = Salva
lists_picker_title = Aggiungi { $handle } alle liste
lists_picker_none = Non hai ancora liste.
lists_delete = Elimina lista
lists_delete_title = Eliminare questa lista?
lists_delete_body = «{ $name }» verrà rimossa. Le persone al suo interno non sono interessate.
lists_by = Lista di { $handle }
lists_tab_posts = Post
lists_tab_members = Membri
lists_tabs = Sezioni della lista
lists_error_title = Impossibile aprire questa lista
lists_posts_label = Post della lista
lists_posts_empty_title = Ancora niente qui
lists_posts_empty_body = I post dei membri di questa lista compaiono qui.
lists_members_label = Membri della lista
lists_members_empty_title = Nessun membro
lists_members_empty_body = Aggiungi persone dai loro profili.
lists_remove = Rimuovi
scheduled_title = Post programmati
scheduled_label = Post programmati
scheduled_status_pending = Programmato
scheduled_status_published = Pubblicato
scheduled_status_failed = Non riuscito
scheduled_cw = Avviso sui contenuti: { $text }
scheduled_poll = Sondaggio con { $count } opzioni, dura { $hours } h
scheduled_change_time = Cambia orario
scheduled_post_now = Pubblica ora
scheduled_remove = Rimuovi
scheduled_window_error = Scegli un orario tra un minuto e 90 giorni da adesso.
scheduled_when = Pubblica il
scheduled_zone = Fuso orario: { $zone }
scheduled_save = Salva
scheduled_posted = Pubblicato.
scheduled_cancel_title = Annullare questo post programmato?
scheduled_cancel_body = Non verrà pubblicato.
scheduled_cancel_confirm = Annulla post
scheduled_empty_title = Niente di programmato
scheduled_empty_body = I post che programmi restano qui fino all’orario stabilito.
profile_message = Messaggio
profile_add_to_lists = Aggiungi alle liste
media_alt = ALT
profile_report = Segnala { $handle }
chart_show_table = Mostra { $title } come tabella
chart_col_day = Giorno
chart_col_hour = Ora (UTC)
chart_empty = { $title }: ancora niente da mostrare.
chart_bar_summary = { $title }: { $total } in totale, il massimo il { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } in totale, { $last } il { $label }.
chart_heat_summary = { $title }: { $total } in totale, picco il { $label } alle { $hour } ({ $peak }).
chart_heat_less = Meno
chart_heat_more = Più
announce_region = Annunci
announce_dismiss = Chiudi l’annuncio
announce_more = Leggi tutto
announce_tone_info = Avviso
announce_tone_success = Buone notizie
announce_tone_warning = Attenzione
admin_stat_people = Persone
admin_stat_people_sub = { $count } attivi questa settimana
admin_stat_posts_today = Post, 24 h
admin_stat_posts_total = { $count } in totale
admin_stat_reports = Segnalazioni aperte
admin_stat_reports_open = Da esaminare
admin_stat_reports_clear = Tutto a posto
admin_stat_signups_today = Iscrizioni oggi
admin_stat_signups_sub = { $count } in 14 giorni
admin_stat_suspended = Sospesi
admin_stat_suspended_sub = account
admin_chart_posts = Post al giorno
admin_chart_signups = Iscrizioni al giorno
admin_chart_reports = Segnalazioni al giorno
admin_chart_heat = Attività per giorno della settimana e ora
nav_admin = Amministrazione
admin_title = Amministrazione
admin_sections = Sezioni di amministrazione
admin_tab_overview = Panoramica
admin_tab_reports = Segnalazioni
admin_tab_users = Persone
admin_tab_instance = Istanza
admin_tab_federation = Federazione
admin_tab_announcements = Annunci
admin_tab_invites = Inviti
admin_tab_filter = Filtro contenuti
admin_tab_audit = Registro di controllo
admin_denied_title = Solo per lo staff
admin_denied_body = Quest’area è riservata agli amministratori e ai moderatori di questa istanza.
admin_denied_home = Torna alla home
admin_unknown = sconosciuto
admin_tag_active = Attivo
admin_tag_suspended = Sospeso
admin_tag_deleted = Eliminato
admin_tag_org = Organizzazione
admin_audit_system = sistema
admin_audit_empty = Ancora nulla di registrato
admin_report_status = Stato della segnalazione
admin_report_open = Aperte
admin_report_resolved = Risolte
admin_reports_empty_open = Nessuna segnalazione aperta
admin_reports_empty_open_body = Quando qualcuno segnala post o account, compaiono qui.
admin_reports_empty_resolved = Ancora nulla di risolto
admin_report_by = segnalato da { $handle }
admin_report_same_target = { $count } segnalazioni aperte
admin_report_media =
    { $count ->
        [one] { $count } allegato
       *[other] { $count } allegati
    }
admin_report_poll = Contiene un sondaggio
admin_report_dismiss = Ignora
admin_report_delete_post = Elimina post
admin_report_suspend = Sospendi account
admin_report_delete_suspend = Elimina e sospendi
admin_report_confirm_title = Procedere?
admin_report_confirm_body = { $action }. L’azione viene registrata nel registro di controllo.
admin_report_confirm = Conferma
admin_report_resolution = { $action } da { $handle } · { $time }
admin_res_dismiss = Ignorata
admin_res_delete_post = Post eliminato
admin_res_suspend_user = Account sospeso
admin_res_delete_and_suspend = Post eliminato e account sospeso
admin_users_search = Cerca persone
admin_users_search_placeholder = Nome o nome utente
admin_users_show = Mostra
admin_users_empty = Nessuna persona trovata
admin_filter_local = Locali
admin_filter_remote = Remoti
admin_filter_staff = Staff
admin_filter_verified = Verificati
admin_filter_suspended = Sospesi
admin_filter_all = Tutti
admin_user_details = Dettagli di { $handle }
admin_user_menu = Gestisci { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } post
    }
admin_user_followers =
    { $count ->
        [one] { $compact } follower
       *[other] { $compact } follower
    }
admin_user_joined = iscritto { $time }
admin_user_active = attivo { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } segnalazione aperta
       *[other] { $compact } segnalazioni aperte
    }
admin_act_verify = Verifica
admin_act_unverify = Rimuovi verifica
admin_act_org = Imposta come organizzazione
admin_act_unorg = Rimuovi stato di organizzazione
admin_act_role_user = Rimuovi ruolo di staff
admin_act_role_moderator = Imposta come moderatore
admin_act_role_admin = Imposta come amministratore
admin_act_suspend = Sospendi
admin_act_unsuspend = Riattiva account
admin_act_purge = Elimina tutti i post
admin_act_rename = Cambia nome utente
admin_act_reset_name = Reimposta nome visualizzato
admin_act_clear_bio = Cancella bio
admin_act_delete = Elimina account
admin_done_verified = { $handle } è verificato
admin_done_unverified = Verifica rimossa
admin_done_org = { $handle } è ora un’organizzazione
admin_done_unorg = Stato di organizzazione rimosso
admin_done_role = Ruolo aggiornato
admin_done_suspended = { $handle } sospeso
admin_done_unsuspended = { $handle } riattivato
admin_done_purged =
    { $count ->
        [one] { $count } post eliminato
       *[other] { $count } post eliminati
    }
admin_done_name_reset = Nome visualizzato reimpostato
admin_done_bio_cleared = Bio cancellata
admin_done_deleted = { $handle } eliminato
admin_done_renamed = Ora è { $handle }
admin_role_phrase_user = un utente normale
admin_role_phrase_moderator = un moderatore
admin_role_phrase_admin = un amministratore
admin_role_title = Cambiare il ruolo di { $handle }?
admin_role_body = Diventerà { $role }.
admin_role_confirm = Cambia ruolo
admin_unorg_title = Rimuovere lo stato di organizzazione a { $handle }?
admin_unorg_body = Tutte le sue affiliazioni terminano.
admin_unorg_confirm = Rimuovi
admin_suspend_title = Sospendere { $handle }?
admin_suspend_body = Verrà disconnesso e i suoi contenuti resteranno nascosti finché non riattivi l’account.
admin_suspend_confirm = Sospendi
admin_purge_title = Eliminare tutti i post di { $handle }?
admin_purge_body = Non si può annullare.
admin_purge_confirm = Elimina tutto
admin_reset_name_title = Reimpostare il nome visualizzato di { $handle }?
admin_reset_name_body = Verrà sostituito con il suo nome utente.
admin_reset_name_confirm = Reimposta
admin_delete_title = Eliminare { $handle }?
admin_delete_body = Post, follow e messaggi vengono rimossi e l’account viene reso anonimo. Non si può annullare.
admin_delete_confirm = Elimina account
admin_rename_title = Cambia il nome utente di { $handle }
admin_rename_note = I link al vecchio nome utente smetteranno di funzionare.
admin_rename_handle = Nuovo nome utente
admin_rename_reset_name = Reimposta anche il nome visualizzato
admin_rename_save = Cambia nome utente
admin_detail_title = Dettagli dell’account
admin_detail_invite = Invito { $code }
admin_detail_posts = Post
admin_detail_followers = Follower
admin_detail_following = Seguiti
admin_detail_open_reports = Segnalazioni aperte
admin_detail_sessions = Sessioni
admin_detail_passkeys = Passkey
admin_detail_tokens = Token API
admin_detail_joined = Iscritto il { $time }
admin_detail_recent_posts = Post recenti
admin_detail_no_posts = Nessun post.
admin_detail_reports = Segnalazioni su questo account
admin_detail_no_reports = Nessuna segnalazione.
admin_detail_history = Cronologia di moderazione
admin_detail_nothing = Ancora niente.
admin_instance_title = Istanza
admin_instance_name = Nome dell’istanza
admin_instance_description = Descrizione
admin_instance_contact = Email di contatto
admin_instance_contact_hint = Mostrata nelle pagine dei Termini e della Privacy
admin_instance_min_age = Età minima
admin_instance_rules = Regole
admin_instance_rules_hint = Una regola per riga
admin_instance_preview = Anteprima dei termini
admin_instance_save = Salva
admin_instance_saved = Impostazioni dell’istanza salvate
admin_instance_template_note = I Termini di servizio e l’Informativa sulla privacy sono modelli che usano questi valori. Falli rivedere per la tua giurisdizione prima di aprire l’istanza al pubblico.
admin_signups_title = Iscrizioni
admin_signups_desc = Chi può creare un account su questa istanza
admin_signups_open = Aperte
admin_signups_open_hint = Chiunque può creare un account.
admin_signups_invite = Solo su invito
admin_signups_invite_hint = Serve un codice invito dalla scheda Inviti.
admin_signups_closed = Chiuse
admin_signups_closed_hint = Nessuno può iscriversi.
admin_filter_title = Filtro contenuti
admin_filter_desc = Niente viene bloccato finché non aggiungi parole qui sotto. Le parole bloccate valgono per nomi utente, nomi visualizzati, località, bio, post e nomi delle liste, comprese le grafie camuffate. I nomi utente riservati come «admin» non possono essere registrati.
admin_filter_blocked = Parole bloccate
admin_filter_blocked_hint = Una per riga, cercate anche dentro le parole
admin_filter_allowed = Parole consentite
admin_filter_allowed_hint = Eccezioni alle parole bloccate, una per riga
admin_filter_reserved = Altri nomi utente riservati
admin_filter_reserved_hint = Uno per riga
admin_filter_save = Salva filtro
admin_filter_saved = Filtro salvato
admin_filter_field_handle = nome utente
admin_filter_field_name = nome
admin_filter_field_location = località
admin_filter_field_bio = bio
admin_scan_title = Scansione degli account
admin_scan_desc = Trova gli account il cui nome utente, nome o bio violano il filtro.
admin_scan_run = Scansiona ora
admin_scan_clean = Nessun account viola il filtro.
admin_scan_found =
    { $count ->
        [one] { $count } account richiede attenzione
       *[other] { $count } account richiedono attenzione
    }
admin_scan_reason = Il campo { $field } contiene { $reason }
admin_ann_title = Nuovo annuncio
admin_ann_desc = Un banner in cima allo schermo di tutti finché non scade o non lo termini. Le persone possono chiuderlo.
admin_ann_text = Messaggio
admin_ann_link = Link
admin_ann_link_hint = Facoltativo: https://… o /percorso
admin_ann_tone = Stile
admin_ann_tone_info = Info
admin_ann_tone_success = Buone notizie
admin_ann_tone_warning = Attenzione
admin_ann_duration = Durata
admin_ann_days =
    { $count ->
        [one] { $count } giorno
       *[other] { $count } giorni
    }
admin_ann_week = 1 settimana
admin_ann_until_ended = Fino alla chiusura
admin_ann_publish = Pubblica
admin_ann_published = Annuncio pubblicato
admin_ann_list = Annunci finora
admin_ann_none = Ancora nessun annuncio.
admin_ann_end = Termina
admin_ann_ended = terminato
admin_ann_expired = scaduto
admin_ann_inactive = Non visibile
admin_ann_until = fino al { $date }
admin_inv_title = Codici invito
admin_inv_desc = Permetti a persone specifiche di iscriversi mentre le iscrizioni sono solo su invito.
admin_inv_note = Nota (per chi è)
admin_inv_uses = Utilizzi
admin_inv_use_count =
    { $count ->
        [one] { $count } utilizzo
       *[other] { $count } utilizzi
    }
admin_inv_expiry = Scadenza
admin_inv_week = Tra 1 settimana
admin_inv_days =
    { $count ->
        [one] Tra { $count } giorno
       *[other] Tra { $count } giorni
    }
admin_inv_never = Mai
admin_inv_create = Crea codice
admin_inv_created = Codice { $code } creato
admin_inv_created_copied = Codice { $code } creato e copiato
admin_inv_list = Codici
admin_inv_none = Ancora nessun codice invito.
admin_inv_state_active = Attivo
admin_inv_state_revoked = Revocato
admin_inv_state_used = Esaurito
admin_inv_state_expired = Scaduto
admin_inv_used = utilizzi: { $uses } su { $max }
admin_inv_expires = scade il { $date }
admin_inv_copy = Copia il codice { $code }
admin_inv_copied = Codice copiato
admin_inv_copy_failed = Impossibile copiare. Il codice è { $code }.
admin_inv_revoke = Revoca
admin_inv_revoke_title = Revocare questo invito?
admin_inv_revoke_body = Chi si è già iscritto con questo codice mantiene il proprio account.
settings_devices = Dispositivi e sessioni
settings_devices_desc = Vedi dove hai effettuato l’accesso ed esci da remoto
settings_passkeys = Passkey
settings_passkeys_desc = Accedi con impronta, volto, blocco schermo o una chiave di sicurezza
settings_affiliation = Affiliazione
settings_affiliation_desc = Le organizzazioni di cui fai parte, o gli account della tua organizzazione
settings_bots = Bot e API
settings_bots_desc = Token API ed etichetta «Automatizzato»
settings_saved = Salvato
passkey_title = Le tue passkey
passkey_desc = Accedi con impronta, volto, blocco schermo o chiave di sicurezza al posto della password. Le passkey non possono essere carpite con il phishing e sul nostro server non viene conservato nulla di segreto.
passkey_none = Ancora nessuna passkey.
passkey_add = Aggiungi una passkey
passkey_added = Passkey «{ $name }» aggiunta
passkey_default_name = Passkey
passkey_added_ago = aggiunta { $time }
passkey_used_ago = ultimo utilizzo { $time }
passkey_never_used = mai usata
passkey_rename = Rinomina
passkey_rename_label = Rinomina la passkey { $name }
passkey_rename_title = Rinomina passkey
passkey_name = Nome
passkey_rename_save = Salva
passkey_remove = Rimuovi
passkey_remove_label = Rimuovi la passkey { $name }
passkey_remove_title = Rimuovere «{ $name }»?
passkey_remove_body = Non potrai più accedere con questa passkey. Potresti anche volerla eliminare dal dispositivo o dal gestore di password.
passkey_removed = Passkey rimossa
passkey_unsupported = Questo browser non può usare le passkey su questo indirizzo.
passkey_needs_domain = Le passkey richiedono che il sito abbia un nome di dominio, non un indirizzo IP.
passkey_already_here = Questo dispositivo ha già una passkey per questo account.
token_scope_read = Lettura
token_scope_write = Pubblicare e interagire
token_scope_messages = Messaggi diretti
token_scope_read_hint = Timeline, notifiche, profili, ricerca e flusso in tempo reale
token_scope_write_hint = Pubblicare, eliminare, mi piace, repost, segnalibri, follow e caricamenti
token_scope_messages_hint = Leggere e inviare messaggi diretti
token_title = Token API
token_desc = I token permettono a script e bot di usare il tuo account tramite l’API. Trattali come password: chi ha un token può agire come te entro i suoi permessi.
token_none = Ancora nessun token.
token_name = Nome del token
token_name_hint = Ad esempio: bot del meteo
token_scopes = Ambiti
token_create = Crea token
token_used_ago = ultimo utilizzo { $time }
token_never_used = mai usato
token_revoke = Revoca
token_revoke_label = Revoca il token { $name }
token_revoke_title = Revocare «{ $name }»?
token_revoke_body = Tutto ciò che lo usa smette di funzionare.
token_revoked = Token revocato
token_secret_title = Il tuo nuovo token
token_secret_warning = Copialo ora. Per sicurezza non verrà mostrato di nuovo.
token_secret_label = Nuovo token API
token_copy = Copia
token_copied = Token copiato
token_copy_failed = Impossibile copiare. Seleziona il token e copialo a mano.
token_done = Fatto
bot_title = Account automatizzato
bot_desc = Mostra l’etichetta «Automatizzato» accanto al nome, così si sa che pubblica un bot.
bot_switch = Questo è un account automatizzato
bot_owner = Gestito da
bot_owner_hint = Il nome utente della persona o dell’organizzazione che lo gestisce. Facoltativo.
bot_owner_save = Salva
bot_docs = Documentazione per sviluppatori
bot_docs_desc = Endpoint, eventi in tempo reale e bot di esempio
aff_yours_title = La tua affiliazione
aff_yours_desc = Un’organizzazione può invitarti a mostrare il suo badge accanto al tuo nome. Nulla cambia finché non accetti, e puoi uscire quando vuoi.
aff_none = Non sei affiliato a nessuna organizzazione.
aff_invites_title = Inviti
aff_accept = Accetta
aff_decline = Rifiuta
aff_leave = Esci
aff_leave_title = Lasciare questa organizzazione?
aff_leave_body = Il suo badge scompare dal tuo profilo.
aff_remove_title = Rimuovere { $handle }?
aff_remove_body = Il tuo badge scompare dal suo profilo.
aff_remove = Rimuovi
aff_remove_label = Rimuovi { $handle }
aff_ended = Affiliazione terminata
aff_declined = Invito rifiutato
aff_accepted = Affiliazione aggiunta
aff_invited = Invito inviato
aff_org_title = I tuoi affiliati
aff_org_desc = Invita le persone a mostrare il tuo badge accanto al loro nome. Devono accettare.
aff_handle = Nome utente
aff_label = Etichetta
aff_label_hint = Facoltativa, ad esempio «Ingegnere»
aff_invite = Invia invito
aff_members_none = Ancora nessun affiliato.
aff_pending = Invitato
aff_not_org = Sei un’organizzazione? Chiedi a un amministratore di questa istanza di contrassegnare il tuo account.
auth_passkey = Accedi con una passkey
auth_or = oppure
auth_invite = Codice invito
auth_invite_hint = Questa istanza accetta nuove persone solo su invito.
auth_invite_required = Inserisci il codice invito che hai ricevuto.
landing_developers = Sviluppatori
landing_invite = Le iscrizioni sono solo su invito. Hai un codice? Crea il tuo account.
dev_title = Sviluppatori
dev_hero = Bot e API
dev_hero_sub = Crea bot, script e integrazioni per { $name }
dev_intro_h = Sviluppa per { $name }
dev_intro_p = Tutto ciò che fanno le app tuita passa da una piccola API JSON, che possono usare anche bot e script. Ti servono un account su { $name } e un token API personale. Non c’è registrazione dell’app né procedure OAuth.
dev_intro_1 = Crea un account per il tuo bot oppure usa il tuo.
dev_intro_2 = In Impostazioni, Bot e API, attiva «Questo è un account automatizzato» così le persone vedono l’etichetta «Automatizzato», e indica chi lo gestisce.
dev_intro_3 = Crea un token con gli ambiti che ti servono e tienilo segreto.
dev_auth_h = Autenticazione
dev_auth_p = Invia il token nell’intestazione Authorization. Ogni richiesta e risposta è in JSON (UTF-8), tranne i caricamenti, che sono multipart.
dev_auth_prefix = Un token è tuita_ seguito da 43 lettere, cifre, trattini e trattini bassi. Viene conservato solo un hash, quindi un token perso non può essere mostrato di nuovo: creane uno nuovo. Un token ha uno o più di questi ambiti:
dev_auth_refused = Un token non può cambiare password, sessioni, token o passkey, non può esportare o eliminare l’account e non può usare l’API di amministrazione. Queste operazioni richiedono un vero accesso dal sito e con un token rispondono 403.
dev_auth_errors = Gli errori tornano in JSON con un messaggio di errore e uno stato 4xx. 401 significa che il token è sbagliato o revocato, 403 un ambito mancante o un percorso rifiutato, e 429 arriva con un’intestazione Retry-After.
dev_post_h = Il tuo primo post
dev_post_p = Carica prima immagini e video con POST /api/v1/media (multipart), poi allegali con il loro id in mediaIds. Una risposta imposta replyTo, una citazione imposta quoteOf.
dev_endpoints_h = Endpoint
dev_endpoints_intro = I percorsi tra parentesi graffe sono segnaposto. Le liste si paginano con ?before= impostato sul valore successivo della pagina precedente; limit va da 1 a 40. La colonna dell’ambito indica l’ambito richiesto dal token.
dev_col_method = Metodo
dev_col_path = Percorso
dev_col_scope = Ambito
dev_col_what = Cosa fa
dev_ep_me = L’account a cui appartiene il token
dev_ep_patch_me = Modifica profilo e impostazioni di quell’account
dev_ep_bot = Attiva o disattiva l’etichetta «Automatizzato» e indica chi gestisce il bot
dev_ep_following = La timeline Home delle persone che segui
dev_ep_foryou = La timeline «Per te»; paginazione con ?cursor=
dev_ep_notifications = Notifiche (?filter=mentions per risposte, menzioni e citazioni)
dev_ep_post = Un post con il suo thread e le risposte
dev_ep_user = Un profilo (utente@dominio per gli account di altri server)
dev_ep_user_posts = I post di una persona (?tab=replies, media o likes; con media anche ?kind=photo o video)
dev_ep_search = Cerca post, persone o media, con operatori come from: e has:media
dev_ep_trends = I tag più usati nell’ultima settimana
dev_ep_stream = Eventi in tempo reale (Server-Sent Events)
dev_ep_create = Scrivi un post
dev_ep_thread = Scrivi da 2 a 25 post come thread
dev_ep_delete = Elimina uno dei tuoi post
dev_ep_like = Metti mi piace a un post (DELETE per annullare). Anche /repost e /bookmark
dev_ep_follow = Segui qualcuno (DELETE per smettere)
dev_ep_media = Carica un’immagine o un video come multipart; la risposta contiene l’id da allegare
dev_ep_schedule = Programma un post per più tardi
dev_ep_report = Segnala un post o un account ai moderatori
dev_ep_conversations = Le tue conversazioni di messaggi diretti
dev_ep_message = Invia un messaggio diretto; il corpo è JSON con un campo text
dev_realtime_h = Eventi in tempo reale
dev_realtime_p = GET /api/v1/stream è un flusso Server-Sent Events. Invia un evento notification per ogni mi piace, repost, follow, risposta, menzione o citazione, un evento message per i messaggi diretti e badge con i conteggi dei non letti. Invia una riga di commento ogni 25 secondi e chiede ai client di riprovare dopo 5 secondi; riconnettiti se la connessione cade.
dev_example_h = Esempio: un bot che risponde alle menzioni
dev_example_node = Node 22 o successivo, nessun pacchetto:
dev_example_python = Lo stesso in Python, usando solo requests:
dev_limits_h = Limiti
dev_limit_rate = Ogni token può fare 600 richieste ogni 10 minuti. Oltre il limite il server risponde 429 con un’intestazione Retry-After.
dev_limit_tokens = Un account può avere fino a 20 token e crearne fino a 20 all’ora.
dev_limit_posting = Pubblicare, seguire, segnalare e le altre azioni hanno limiti propri per account.
dev_rules_h = Regole per i bot
dev_rule_1 = Attiva l’etichetta «Automatizzato» e indica chi gestisce il bot.
dev_rule_2 = Rispondi o scrivi solo a chi lo ha chiesto: ha menzionato il bot, lo segue o gli ha scritto per primo. Non inviare menzioni o messaggi diretti non richiesti.
dev_rule_3 = Vai piano: resta ben al di sotto dei limiti indicati sopra.
dev_rule_4 = Rispetta i Termini e le regole dell’istanza. I moderatori possono sospendere i bot come qualsiasi account.
admin_act_log_admin_bootstrap = Primo amministratore configurato
admin_act_log_announcement_create = Annuncio pubblicato
admin_act_log_announcement_end = Annuncio terminato
admin_act_log_instance_settings = Impostazioni dell’istanza modificate
admin_act_log_invite_create = Invito creato
admin_act_log_invite_revoke = Invito revocato
admin_act_log_post_delete = Post eliminato
admin_act_log_report_dismiss = Segnalazione ignorata
admin_act_log_user_delete = Account eliminato
admin_act_log_user_org = Segnato come organizzazione
admin_act_log_user_unorg = Stato di organizzazione rimosso
admin_act_log_user_purge_posts = Tutti i post eliminati
admin_act_log_user_rename = Account rinominato
admin_act_log_user_role = Ruolo cambiato
admin_act_log_user_suspend = Account sospeso
admin_act_log_user_unsuspend = Account riattivato
admin_act_log_user_unverify = Verifica rimossa
admin_act_log_user_verify = Account verificato
admin_act_log_domain_block = Istanza bloccata
admin_act_log_domain_unblock = Istanza sbloccata
admin_act_log_federation_retry = Nuovo tentativo di consegna richiesto
admin_fed_stat_domains = Istanze note
admin_fed_stat_domains_sub = viste da questa istanza
admin_fed_stat_accounts = Account remoti
admin_fed_stat_accounts_sub = noti da altrove
admin_fed_stat_posts = Post remoti
admin_fed_stat_posts_sub = archiviati localmente
admin_fed_stat_queue = In coda
admin_fed_stat_queue_sub = consegne in attesa
admin_fed_queue_title = Coda di consegna
admin_fed_queue_desc = Attività firmate in attesa di raggiungere altre istanze. Quelle fallite vengono ritentate con ritardi crescenti e scartate dopo 12 tentativi.
admin_fed_queue_empty = Niente in attesa. Ogni consegna è stata inviata.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } consegna in attesa
       *[other] { $compact } consegne in attesa
    }
admin_fed_queue_due = { $count } da inviare ora
admin_fed_queue_oldest = La più vecchia in coda dal { $date }
admin_fed_retry = Riprova ora
admin_fed_retry_done =
    { $count ->
        [one] { $compact } consegna verrà ritentata ora.
       *[other] { $compact } consegne verranno ritentate ora.
    }
admin_fed_failing = Istanze con errori
admin_fed_failing_none = Nessuna istanza ha errori.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } consegna
       *[other] { $compact } consegne
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } tentativo
       *[other] { $compact } tentativi
    }
admin_fed_failing_next = prossimo tentativo il { $date }
admin_fed_failing_error = Ultimo errore
admin_fed_failing_no_error = Nessun errore registrato
admin_fed_actor_title = Attore dell’istanza
admin_fed_actor_desc = Le altre istanze verificano con questa chiave le richieste firmate da questa istanza.
admin_fed_actor_id = Attore
admin_fed_actor_key = ID della chiave
admin_fed_actor_fingerprint = Impronta SHA-256
admin_fed_actor_copy = Copia impronta
admin_fed_actor_copied = Impronta copiata
admin_fed_actor_copy_failed = Impossibile copiare. L’impronta è { $fingerprint }
admin_fed_blocks_title = Istanze bloccate
admin_fed_blocks_desc = «Silenzia» nasconde gli account di un’istanza dalle timeline pubbliche e dalla ricerca. «Sospendi» li nasconde ovunque, scarta ciò che inviano e interrompe le consegne verso di loro.
admin_fed_domain = Dominio
admin_fed_domain_hint = example.social, o il link a una sua pagina qualsiasi
admin_fed_severity = Gravità
admin_fed_silence = Silenzia
admin_fed_suspend = Sospendi
admin_fed_reason = Motivo
admin_fed_reason_hint = Facoltativo, visibile solo allo staff
admin_fed_block = Blocca istanza
admin_fed_blocked = { $domain } è bloccata
admin_fed_none = Nessuna istanza bloccata.
admin_fed_list = Blocchi
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } account noto
       *[other] { $compact } account noti
    }
admin_fed_row_no_reason = Nessun motivo indicato
admin_fed_row_changed = { $domain } ora è impostata su { $severity }
admin_fed_remove = Rimuovi
admin_fed_remove_label = Rimuovi il blocco su { $domain }
admin_fed_remove_title = Rimuovere il blocco su { $domain }?
admin_fed_remove_body = I suoi account e post tornano visibili e le consegne verso di essa riprendono.
admin_fed_removed = { $domain } non è più bloccata
remote_domain_title = Account su { $domain }
remote_view_original = Vedi su { $domain }
visibility_unlisted = Non in elenco
visibility_unlisted_hint = Non mostrato nelle timeline pubbliche
visibility_followers = Solo follower
visibility_followers_hint = Solo i follower possono vederlo
visibility_direct = Diretto
visibility_direct_hint = Solo le persone menzionate possono vederlo
follow_requested = Richiesto
follow_cancel_request = Annulla richiesta di follow
follow_requested_toast = Richiesta di follow inviata a { $handle }
follow_cancelled_toast = Richiesta di follow annullata
profile_locked = Approva i follower
profile_locked_hint = Le richieste di follow richiedono l’approvazione di questo account.
profile_original = Vedi il profilo su { $domain }
profile_remote_note = Una copia da { $domain }. Conteggi e post sono quelli visti da questa istanza.
remote_lookup_loading = Ricerca di { $handle } su { $domain }…
remote_lookup_notfound_title = Account inesistente
remote_lookup_notfound_body = { $handle } non esiste, oppure { $domain } non lo condivide.
remote_lookup_failed_title = Impossibile caricare { $handle }
explore_lookup = Ricerca di { $target }…
explore_lookup_empty_body = Account o post non trovato. L’altra istanza potrebbe essere offline o non condividerlo.
language_label = Lingua
oauth_title = Autorizza un’app
oauth_loading = Controllo della richiesta…
oauth_wants = Questa app vuole usare il tuo account { $handle }.
oauth_name_warning = Chiunque può scegliere un nome per l’app. Approva solo le app di cui ti fidi.
oauth_website = Sito web
oauth_allow = Potrà
oauth_scope_read = Leggere il tuo profilo, i post, i seguiti, le liste, le timeline e le notifiche, e i messaggi diretti non cifrati end-to-end.
oauth_scope_write = Pubblicare, modificare ed eliminare per te, mettere mi piace, seguire, bloccare, silenziare e cambiare profilo e impostazioni.
oauth_scope_messages = Leggere e inviare i tuoi messaggi diretti.
oauth_push_note = Le notifiche push non sono supportate. Un’app che le richiede non le riceve.
oauth_redirect = Dopo la tua risposta verrai reindirizzato a
oauth_redirect_oob = L’app non riceve alcun reindirizzamento. Qui viene mostrato un codice da incollare al suo interno.
oauth_approve = Autorizza
oauth_deny = Rifiuta
oauth_switch = Usa un altro account
oauth_sent_approved = Autorizzata. Ti stiamo riportando su { $host }. Se non succede nulla, chiudi questa scheda.
oauth_sent_denied = Rifiutata. Ti stiamo riportando su { $host }. Se non succede nulla, chiudi questa scheda.
oauth_denied = Rifiutata. Non è stato condiviso nulla con l’app.
oauth_code_title = Codice di autorizzazione
oauth_code_body = Incolla questo codice nell’app. Funziona una sola volta e scade tra 10 minuti.
oauth_code_copy = Copia codice
oauth_code_copied = Codice copiato
oauth_code_copy_failed = Impossibile copiare. Seleziona il codice e copialo a mano.
oauth_error_title = Questa richiesta non può essere autorizzata
oauth_error_note = Non è stato inviato nulla all’app.
oauth_error_invalid_request = La richiesta non è valida, oppure chiede di tornare a un indirizzo che l’app non ha registrato. Torna all’app e riprova.
oauth_error_invalid_client = Questa app non è registrata qui.
oauth_error_invalid_scope = L’app chiede un accesso che tuita non offre, o più di quello per cui è registrata.
oauth_error_unsupported_response_type = L’app chiede un tipo di accesso che tuita non supporta.
oauth_error_rate = Hai risposto a troppe richieste. Attendi qualche minuto e riprova.
oauth_error_unreachable = Impossibile raggiungere tuita. Controlla la connessione e riprova.
oauth_home = Vai a tuita
share_send_chat = Invia in chat
share_download = Scarica i contenuti
share_download_n = Scarica il contenuto { $number }
share_chat_title = Invia il post in chat
share_chat_sent = Inviato a { $handle }
hover_card_label = Scheda del profilo di { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Tipo di contenuto
profile_media_all = Tutti i contenuti
profile_media_photos = Foto
profile_media_videos = Video
profile_empty_videos_title = Ancora nessun video
profile_empty_videos_body = I post con video compariranno qui.
account_add_existing = Aggiungi un account esistente
rail_label = Ricerca e suggerimenti
rail_search_submit = Cerca
rail_show_more = Mostra altro
rail_footer = Link
chat_new = Nuova chat
chat_search = Cerca conversazioni
chat_search_none = Nessuna conversazione corrisponde alla ricerca.
chat_start_title = Inizia conversazione
chat_start_body = Scegli una conversazione dall’elenco o iniziane una nuova.
explore_sections = Sezioni di Esplora
explore_see_more = Mostra altro
history_title = Cronologia
history_tabs = Sezioni della cronologia
history_likes_label = Post piaciuti
video_play = Riproduci
video_pause = Pausa
video_mute = Disattiva audio
video_unmute = Attiva audio
video_volume = Volume
video_seek = Cerca nel video
video_position = { $current } di { $total }
video_speed = Velocità di riproduzione
video_fullscreen = Schermo intero
video_exit_fullscreen = Esci dallo schermo intero
video_pip = Picture in picture
video_exit_pip = Esci da picture in picture
video_error = Impossibile riprodurre questo video.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Nascondi pannello
lightbox_show_panel = Mostra pannello
lightbox_view_post = Vedi post
lightbox_panel = Post e risposte
emotepick_tabs = Set di emoji
emotepick_tab_emoji = Standard
emotepick_tab_custom = Personalizzate
admin_emote_tab = Emote
admin_emote_title = Emote personalizzate
admin_emote_desc = Le emote appartengono all’intera istanza. Chiunque può usarle come :shortcode: nei post, nei nomi e nei messaggi. Uno shortcode non può essere rinominato; elimina l’emote e caricala di nuovo.
admin_emote_shortcode = Shortcode
admin_emote_shortcode_hint = Da 2 a 32 lettere, numeri o trattini bassi. Mostrato come :shortcode:.
admin_emote_invalid = Usa solo lettere, numeri e trattini bassi, da 2 a 32 caratteri.
admin_emote_category = Categoria
admin_emote_category_hint = Facoltativa. Raggruppa le emote nel selettore.
admin_emote_file = Immagine dell’emote
admin_emote_file_hint = PNG, JPEG, WebP o GIF, fino a 256 KB. Trascina un file qui o sceglilo.
admin_emote_file_type = Quel file non è un’immagine PNG, JPEG, WebP o GIF.
admin_emote_file_size = L’immagine è troppo grande (max 256 KB).
admin_emote_choose_file = Scegli immagine
admin_emote_change_file = Cambia immagine
admin_emote_upload = Carica emote
admin_emote_upload_failed = Impossibile caricare l’emote. Riprova.
admin_emote_uploaded = Aggiunta :{ $shortcode }:.
admin_emote_list = Emote di questa istanza
admin_emote_none = Ancora nessuna emote personalizzata.
admin_emote_no_category = Nessuna categoria
admin_emote_category_for = Categoria di :{ $shortcode }:
admin_emote_saved = Categoria salvata.
admin_emote_delete = Elimina :{ $shortcode }:
admin_emote_delete_title = Eliminare questa emote?
admin_emote_delete_body = :{ $shortcode }: smette di essere mostrata come immagine ovunque e appare come testo semplice. I post che la usavano mantengono il testo.
admin_emote_delete_confirm = Elimina emote
admin_emote_deleted = Emote eliminata.
admin_act_log_emoji_create = Emote aggiunta
admin_act_log_emoji_update = Emote modificata
admin_act_log_emoji_delete = Emote eliminata
e2ee_settings = Messaggi cifrati
e2ee_settings_desc = Frase segreta, questo dispositivo e reimpostazione
e2ee_status_label = Stato
e2ee_status_none = Disattivati. Attivali per inviare e leggere messaggi cifrati end-to-end.
e2ee_status_locked = Bloccati su questo dispositivo. Inserisci la frase segreta per leggere qui i tuoi messaggi.
e2ee_status_ready = Attivi e sbloccati su questo dispositivo.
e2ee_keys_error = Impossibile controllare le tue chiavi di cifratura.
e2ee_setup_title = Attiva i messaggi cifrati
e2ee_setup_body = I messaggi alle persone di questo server vengono cifrati nel tuo browser. Il server conserva solo testo che non può leggere. Una frase segreta ti permette di sbloccare i messaggi su un altro dispositivo.
e2ee_setup_warning = La frase segreta non si può recuperare. Se la dimentichi puoi reimpostare la cifratura, ma i messaggi precedenti andranno persi.
e2ee_setup_submit = Attiva
e2ee_setup_done = I messaggi cifrati sono attivi
e2ee_unlock_title = Sblocca i messaggi cifrati
e2ee_unlock_body = Inserisci la frase segreta per leggere e inviare messaggi cifrati su questo dispositivo.
e2ee_unlock_submit = Sblocca
e2ee_unlock_wrong = La frase segreta non è corretta.
e2ee_unlock_done = Sbloccati
e2ee_passphrase = Frase segreta
e2ee_passphrase_confirm = Conferma la frase segreta
e2ee_passphrase_hint = Almeno 10 caratteri.
e2ee_passphrase_short = Usa almeno 10 caratteri.
e2ee_passphrase_mismatch = Le frasi segrete non coincidono.
e2ee_failed = Qualcosa è andato storto. Riprova.
e2ee_unsupported = Questo browser non supporta i messaggi cifrati.
e2ee_forget = Dimentica su questo dispositivo
e2ee_forget_hint = Rimuove da questo browser la tua chiave e i contatti di cui ti fidi. La frase segreta li sblocca di nuovo.
e2ee_forget_done = Dimenticato su questo dispositivo
e2ee_reset = Reimposta la cifratura
e2ee_reset_hint = Crea una nuova chiave. I messaggi che hai inviato e ricevuto prima diventano illeggibili.
e2ee_reset_title = Reimpostare la cifratura?
e2ee_reset_body = I messaggi precedenti diventano illeggibili, per te e per le persone a cui hai scritto, e non si possono recuperare. Ai tuoi contatti verrà chiesto di fidarsi della tua nuova chiave. Scegli una nuova frase segreta.
e2ee_reset_submit = Reimposta
e2ee_reset_done = La cifratura è stata reimpostata
e2ee_note_sealed = Cifrato end-to-end
e2ee_note_remote = Non cifrato end-to-end: questa persona è su un altro server.
e2ee_note_plain = Non cifrato end-to-end.
e2ee_plain_mark = Non cifrato
e2ee_preview = Messaggio cifrato
e2ee_line_sealed = Messaggio cifrato
e2ee_line_own_key = Inviato a una chiave che non hai più
e2ee_line_peer_key = Inviato con una chiave di cui non ti sei ancora fidato
e2ee_line_unreadable = Impossibile aprirlo
e2ee_line_unsupported = Questo browser non può aprire i messaggi cifrati
e2ee_block_peer_off = { $handle } non ha ancora attivato i messaggi cifrati.
e2ee_block_self_off = Attiva prima i messaggi cifrati nelle Impostazioni.
e2ee_block_unlock = Sblocca prima i messaggi cifrati nelle Impostazioni.
e2ee_block_downgrade = { $handle } ha già usato i messaggi cifrati. Questa conversazione non tornerà ai messaggi non cifrati.
e2ee_block_changed = Fidati della nuova chiave di { $handle } per scrivere a questa persona.
e2ee_block_error = Impossibile controllare le tue chiavi di cifratura.
e2ee_changed_title = La chiave di cifratura di { $name } è cambiata
e2ee_changed_body = Questa persona potrebbe aver reimpostato la cifratura o cambiato dispositivo, oppure qualcuno potrebbe essersi messo in mezzo. Confronta il nuovo numero di sicurezza con lei prima di fidarti. Finché non lo fai, non puoi inviare messaggi.
e2ee_changed_old = Numero di sicurezza di cui ti fidavi
e2ee_changed_new = Nuovo numero di sicurezza
e2ee_trust = Fidati della nuova chiave
e2ee_safety_open = Numero di sicurezza
e2ee_safety_title = Numero di sicurezza
e2ee_safety_body = Confronta questo numero con { $name } di persona o in una chiamata, non in questa chat. Se coincide su entrambi gli schermi, nessuno si è messo in mezzo.

## 0031
composer_upload_failed = Caricamento non riuscito. Riprova.
composer_send_failed = Impossibile raggiungere tuita. Controlla la connessione e riprova.
composer_open_failed = Impossibile aprire l’editor dei post. Riprova.
