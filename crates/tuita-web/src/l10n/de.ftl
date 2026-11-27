app_name = tuita
loading = Wird geladen
close = Schließen
dismiss = Schließen
notifications = Mitteilungen
avatar_more = { $count } weitere
char_counter_left =
    { $count ->
        [one] Noch { $count } Zeichen
       *[other] Noch { $count } Zeichen
    }
char_counter_over =
    { $count ->
        [one] { $count } Zeichen über dem Limit
       *[other] { $count } Zeichen über dem Limit
    }
not_found_title = Hier ist nichts
not_found_body = Diese Seite gibt es nicht oder sie ist umgezogen.
not_found_back = Zurück
tagline = Jeder Beitrag in zwei Tinten.
error_title = Etwas ist schiefgelaufen
cancel = Abbrechen
retry = Erneut versuchen
back = Zurück
view = Ansehen
follow = Folgen
following = Folge ich
follows_you = Folgt dir
verified = Verifiziertes Konto
bot_account = Automatisiertes Konto
bot_label = Bot
link_copied = Link in die Zwischenablage kopiert
text_copied = Text kopiert
time_now = jetzt
time_minutes = { $n } Min.
time_hours = { $n } Std.
time_days = { $n } T.
nav_main = Hauptmenü
nav_home = Startseite
nav_explore = Entdecken
nav_notifications = Mitteilungen
nav_bookmarks = Lesezeichen
nav_profile = Profil
nav_settings = Einstellungen
nav_compose = Posten
nav_menu = Menü
badge_unread = { $count } ungelesen
account_menu = Konten
account_switch = Konto wechseln
account_switch_to = Zu { $handle } wechseln
account_add = Konto hinzufügen
account_logout = { $handle } abmelden
account_logout_all = Von allen Konten abmelden
home_feed = Timeline
home_for_you = Für dich
home_following = Folge ich
home_empty_title = Hier ist noch nichts
home_empty_body = Poste als Erstes etwas.
home_following_empty_title = In deinem Kreis ist es ruhig
home_following_empty_body = Folge Leuten, um diese Timeline mit ihren Beiträgen und Reposts zu füllen.
home_find_people = Leute finden
new_posts =
    { $count ->
        [one] { $count } neuen Beitrag anzeigen
       *[other] { $count } neue Beiträge anzeigen
    }
new_posts_many = Über 99 neue Beiträge anzeigen
composer_title = Neuer Beitrag
composer_reply_title = Antworten
composer_quote_title = Beitrag zitieren
composer_placeholder = Was gibt’s Neues?
composer_reply_placeholder = Antwort posten
composer_text_label = Beitragstext
composer_replying_to = Antwort an { $handle }
composer_cw_label = Inhaltswarnung
composer_cw_placeholder = Inhaltswarnung, z. B. Spoiler
composer_cw_toggle = Inhaltswarnung hinzufügen
composer_sensitive_toggle = Medien als sensibel markieren
composer_options = Beitragsoptionen
composer_policy_label = Wer antworten kann
composer_suggestions = Vorschläge
composer_tag_posts =
    { $count ->
        [one] { $compact } Beitrag
       *[other] { $compact } Beiträge
    }
composer_post = Posten
composer_reply = Antworten
policy_everyone = Alle können antworten
policy_following = Leute, denen du folgst, können antworten
policy_mentioned = Nur von dir Erwähnte können antworten
post_sent = Beitrag gesendet
post_by = Beitrag von { $name }
post_reposted_by = { $name } hat repostet
post_pinned = Angehefteter Beitrag
post_replying_to = Antwort an { $handle }
post_replying_unavailable = Antwort auf einen nicht verfügbaren Beitrag
post_unavailable = Dieser Beitrag ist nicht verfügbar.
post_menu = Mehr
post_copy_link = Link kopieren
post_copy_text = Text kopieren
post_pin = Ans Profil anheften
post_unpin = Vom Profil lösen
post_pinned_toast = An dein Profil angeheftet
post_unpinned = Von deinem Profil gelöst
post_delete = Beitrag löschen
post_delete_title = Beitrag löschen?
post_delete_body = Das lässt sich nicht rückgängig machen. Der Beitrag wird aus deinem Profil, aus den Timelines deiner Follower und aus der Suche entfernt.
post_deleted = Beitrag gelöscht
post_view_profile = { $handle } ansehen
post_mute = { $handle } stummschalten
post_block = { $handle } blockieren
post_reply_label = Antworten ({ $count })
post_reply_following_only = Nur Leute, denen die Autorin oder der Autor folgt, können antworten.
post_reply_mentioned_only = Nur in diesem Beitrag erwähnte Leute können antworten.
post_repost_menu = Repost-Optionen
post_repost = Reposten
post_undo_repost = Repost rückgängig machen
post_quote = Zitieren
post_reposts_label = Reposts und Zitate ({ $count })
post_like_label = Gefällt mir ({ $count })
post_bookmark_label = Lesezeichen
post_share = Teilen oder Link kopieren
cw_show = Anzeigen
cw_hide = Ausblenden
cw_label = Inhaltswarnung: { $reason }
muted_toast = { $handle } stummgeschaltet
unmuted_toast = Stummschaltung von { $handle } aufgehoben
blocked_toast = { $handle } blockiert
unblocked_toast = Blockierung von { $handle } aufgehoben
block_title = { $handle } blockieren?
block_body = Die Person kann dir nicht folgen und du siehst ihre Beiträge nicht. Sie wird nicht benachrichtigt.
block_confirm = Blockieren
notify_like = @{ $handle } gefällt dein Beitrag
notify_repost = @{ $handle } hat deinen Beitrag repostet
notify_reply = @{ $handle } hat dir geantwortet
notify_mention = @{ $handle } hat dich erwähnt
notify_quote = @{ $handle } hat deinen Beitrag zitiert
notify_follow = @{ $handle } folgt dir jetzt
notify_follow_request = @{ $handle } möchte dir folgen
notify_poll = Eine Umfrage, an der du teilgenommen hast, ist beendet
notify_affiliation = @{ $handle } lädt dich in die Organisation ein
shortcuts_title = Tastenkürzel
shortcut_group_navigation = Navigation
shortcut_group_actions = Aktionen
shortcut_or = /
shortcut_then = dann
shortcut_plus = +
shortcut_next_prev = Nächster / vorheriger Beitrag
shortcut_open = Beitrag öffnen
shortcut_search = Suchen
shortcut_go_home = Startseite
shortcut_go_explore = Entdecken
shortcut_go_notifications = Mitteilungen
shortcut_go_bookmarks = Lesezeichen
shortcut_go_profile = Dein Profil
shortcut_go_settings = Einstellungen
shortcut_new_post = Neuer Beitrag
shortcut_like = Gefällt mir
shortcut_reply = Antworten
shortcut_repost = Reposten oder zitieren
shortcut_bookmark = Lesezeichen
shortcut_send = Beitrag senden
shortcut_help = Diese Hilfe anzeigen
auth_login_title = Anmelden
auth_add_title = Weiteres Konto hinzufügen
auth_add_sub = Melde dich an, um auf diesem Gerät zwischen bis zu { $max } Konten zu wechseln, ohne dich abzumelden.
auth_signup_title = Konto erstellen
auth_handle = Nutzername
auth_password = Passwort
auth_display_name = Anzeigename
auth_handle_hint = 3–15 Buchstaben, Ziffern oder Unterstriche
auth_password_hint = Mindestens 8 Zeichen
auth_consent_pre = Ich bin mindestens { $minAge } Jahre alt und stimme den{" "}
auth_consent_terms = Nutzungsbedingungen
auth_consent_and = {" "}und der{" "}
auth_consent_privacy = Datenschutzerklärung
auth_consent_end = {" "}zu.
auth_consent_required = Bitte akzeptiere die Nutzungsbedingungen und die Datenschutzerklärung.
auth_signin = Anmelden
auth_signin_add = Anmelden und wechseln
auth_create = Konto erstellen
auth_to_signup = Neu hier? Konto erstellen
auth_to_login = Schon ein Konto? Anmelden
auth_closed = Die Registrierung ist auf dieser Instanz derzeit geschlossen.
landing_nav = Fußzeile
landing_rules = Hausregeln
landing_stats = Gerade jetzt
landing_people = Leute
landing_posts = Beiträge
landing_clock = Deine Ortszeit
landing_trends = Im Trend
landing_trend =
    { $count ->
        [one] { $count } Beitrag
       *[other] { $count } Beiträge
    }
landing_join = { $name } beitreten
landing_join_sub = Erstelle ein Konto und poste in zwei Tinten.
landing_closed = Die Registrierung ist derzeit geschlossen. Schon ein Konto? Anmelden.
landing_terms = AGB
landing_privacy = Datenschutz
landing_source = Quellcode
landing_powered = Betrieben mit tuita { $version }
docs_terms_title = Nutzungsbedingungen
docs_privacy_title = Datenschutzerklärung
docs_updated = Zuletzt aktualisiert: { $date }
docs_date = 5. November 2026
docs_see_privacy = Siehe auch die Datenschutzerklärung.
docs_see_terms = Siehe auch die Nutzungsbedingungen.
docs_contact_fallback = die Administratoren der Instanz (per Direktnachricht oder Meldung)
docs_terms_agreement_h = 1. Zustimmung
docs_terms_agreement_p = Diese Nutzungsbedingungen („Bedingungen“) regeln deine Nutzung von { $name }, der tuita-Instanz unter { $domain } („der Dienst“, „wir“). Mit der Erstellung eines Kontos oder der Nutzung des Dienstes stimmst du diesen Bedingungen und unserer Datenschutzerklärung zu. Wenn du nicht zustimmst, nutze den Dienst nicht.
docs_terms_eligibility_h = 2. Wer den Dienst nutzen darf
docs_terms_eligibility_p = Du musst mindestens { $minAge } Jahre alt sein oder, falls höher, das Mindestalter für die digitale Einwilligung an deinem Wohnort erreicht haben. Du musst eine verbindliche Vereinbarung eingehen können und darfst nach geltendem Recht nicht von der Nutzung des Dienstes ausgeschlossen sein.
docs_terms_account_h = 3. Dein Konto
docs_terms_account_1 = Halte dein Passwort geheim. Du bist für die Aktivität auf deinem Konto verantwortlich.
docs_terms_account_2 = Dein Nutzername darf keine andere Person oder Organisation nachahmen und niemanden darüber täuschen, wer du bist.
docs_terms_account_3 = Sag uns umgehend Bescheid, wenn du vermutest, dass dein Konto kompromittiert wurde. Andere Sitzungen kannst du unter Einstellungen → Konto abmelden.
docs_terms_content_h = 4. Deine Inhalte
docs_terms_content_p1 = Du behältst die Rechte an dem, was du postest. Du räumst uns eine weltweite, nicht exklusive, kostenlose Lizenz ein, deine Inhalte ausschließlich zum Betrieb und zur Verbesserung des Dienstes zu speichern, anzuzeigen, zu vervielfältigen und zu verbreiten. Dazu gehört das Senden an föderierte tuita-Instanzen (siehe Abschnitt 6). Diese Lizenz endet, wenn du die Inhalte oder dein Konto löschst, mit Ausnahme von Kopien, die bereits an andere Instanzen geliefert oder von anderen Personen gespeichert wurden.
docs_terms_content_p2 = Du bist für das verantwortlich, was du postest. Poste nur Inhalte, die du teilen darfst.
docs_terms_rules_h = 5. Community-Regeln
docs_terms_rules_lead = Bei der Nutzung von { $name } stimmst du zu, weder gegen das Gesetz noch gegen diese Regeln zu verstoßen:
docs_terms_rules_p = Du stimmst außerdem zu, den Dienst nicht anzugreifen, zum Beispiel durch das Umgehen von Sicherheitsmaßnahmen, massenhaftes Auslesen, das Versenden von Schadsoftware oder Überlastung.
docs_terms_federation_h = 6. Föderation
docs_terms_federation_p = { $name } kann sich mit anderen unabhängigen tuita-Instanzen verbinden („föderieren“), die die Administratoren freigeben. Wenn du mit Leuten auf einer anderen Instanz interagierst, werden dein öffentliches Profil, deine öffentlichen Beiträge und die Interaktion (zum Beispiel ein Follow, Like, Repost, eine Antwort oder Direktnachricht) an diese Instanz gesendet. Andere Instanzen werden von anderen Personen nach eigenen Bedingungen betrieben. Wir können Inhalte nicht kontrollieren oder löschen, sobald eine andere Instanz sie hat, senden ihr aber Löschanfragen, wenn du etwas löschst.
docs_terms_moderation_h = 7. Moderation und Durchsetzung
docs_terms_moderation_p = Moderatorinnen und Moderatoren können Inhalte entfernen, Funktionen einschränken oder Konten sperren, die gegen diese Bedingungen oder das Gesetz verstoßen. Wenn möglich, teilen wir dir den Grund mit. Jeder kann einen Beitrag oder ein Konto melden. Meldungen gehen an die Moderation dieser Instanz. Wir können jederzeit die Föderation mit einer Instanz beenden.
docs_terms_termination_h = 8. Austritt
docs_terms_termination_p = Du kannst jederzeit unter Einstellungen → Konto deine Daten exportieren oder dein Konto löschen. Wir können deinen Zugang sperren oder beenden, wenn du ernsthaft oder wiederholt gegen diese Bedingungen verstößt. Wir können den Dienst auch einstellen und versuchen, dies rechtzeitig anzukündigen.
docs_terms_disclaimers_h = 9. Haftungsausschluss und Haftung
docs_terms_disclaimers_p = Der Dienst wird im gesetzlich größtmöglichen Umfang „wie besehen“ und „wie verfügbar“ ohne jegliche Gewährleistung bereitgestellt. Wir haften nicht für mittelbare, zufällige oder Folgeschäden oder für Inhalte, die Nutzer oder andere Instanzen posten. Nichts in diesen Bedingungen beschränkt eine Haftung, die gesetzlich nicht beschränkt werden kann, oder deine gesetzlichen Verbraucherrechte.
docs_terms_changes_h = 10. Änderungen
docs_terms_changes_p = Wir können diese Bedingungen aktualisieren. Bei wesentlichen Änderungen informieren wir dich in der App, bevor sie in Kraft treten. Wenn du den Dienst danach weiter nutzt, akzeptierst du die neuen Bedingungen.
docs_terms_contact_h = 11. Kontakt
docs_terms_contact_lead = Fragen zu diesen Bedingungen:
docs_privacy_summary_h = Die Kurzfassung
docs_privacy_summary_1 = Keine Werbung, keine Tracking-Skripte, kein Verkauf von Daten.
docs_privacy_summary_2 = Ein Cookie, damit du angemeldet bleibst. Design-Einstellungen bleiben in deinem Browser.
docs_privacy_summary_3 = Öffentliche Beiträge sind öffentlich und können föderierte tuita-Instanzen erreichen.
docs_privacy_summary_4 = Direktnachrichten zwischen Konten auf diesem Server sind Ende-zu-Ende-verschlüsselt, sobald beide Personen es einschalten: Sie werden als Text gespeichert, den der Server nicht lesen kann, sodass ein Datenbank-Leak oder ein Blick auf die gespeicherten Daten nichts preisgibt. Nachrichten an Leute auf anderen Servern und Nachrichten, die vor dem Einschalten der Verschlüsselung gesendet wurden, sind auf diese Weise nicht verschlüsselt und Administratoren können sie technisch lesen. Dieser Server liefert außerdem die App aus, die verschlüsselt; ein Administrator, der diese App verändert, könnte daher mitlesen, was du eingibst.
docs_privacy_summary_5 = Du kannst deine Daten jederzeit exportieren oder löschen.
docs_privacy_controller_h = 1. Wer wir sind
docs_privacy_controller_p = Diese Erklärung gilt für { $name } unter { $domain }, betrieben von den Administratoren der Instanz. Sie entscheiden, wie deine Daten verarbeitet werden.
docs_privacy_collect_h = 2. Was wir erheben
docs_privacy_collect_1 = Kontodaten: Nutzername, Anzeigename, Passwort (nur als gesalzener argon2id-Hash gespeichert) und deine Profilangaben: Bio, Ort, Website, Avatar, Banner, Form und Farbe.
docs_privacy_collect_2 = Inhalte: Beiträge, Bilder, Antworten, Likes, Reposts, Lesezeichen, Follows, Blockierungen, Stummschaltungen und Meldungen.
docs_privacy_collect_3 = Direktnachrichten: Sie werden auf diesem Server gespeichert, damit sie dir und der anderen Person zugestellt werden können. Eine Ende-zu-Ende-verschlüsselte Nachricht wird als Geheimtext gespeichert, den wir nicht lesen können; wer sie gesendet hat, an wen und wann, bleibt für uns sichtbar. Andere Nachrichten werden als Klartext gespeichert. Dein Verschlüsselungsschlüssel wird hier nur in einer durch deine Passphrase gesperrten Form gespeichert, die wir nie sehen.
docs_privacy_collect_4 = Sitzungsdaten: für jedes angemeldete Gerät der User-Agent des Browsers, die IP-Adresse und die Zeit der letzten Nutzung, damit du Sitzungen prüfen und abmelden kannst.
docs_privacy_collect_5 = Kurzlebige Sicherheitsdaten: IP-Adressen, die für einige Minuten im Arbeitsspeicher bleiben, um Anfragen zu begrenzen.
docs_privacy_collect_p = Wir erheben weder deine E-Mail-Adresse noch Telefonnummer, Kontakte oder deinen Standort, es sei denn, du trägst sie in dein Profil ein.
docs_privacy_use_h = 3. Wie wir sie verwenden
docs_privacy_use_p = Nur um den Dienst zu betreiben: dein Profil und deine Beiträge anzeigen, Timelines und Mitteilungen erstellen, Nachrichten zustellen, Konten schützen, Missbrauch verhindern und die Bedingungen durchsetzen. Die Timeline „Für dich“ sortiert Beiträge nach Likes, Reposts, Antworten und Alter. Sie erstellt kein Profil von dir.
docs_privacy_visibility_h = 4. Was andere sehen können
docs_privacy_visibility_p = Dein Profil, deine Beiträge, Antworten, Reposts, Likes, Follower und die Konten, denen du folgst, sind für alle auf dieser Instanz Angemeldeten und für föderierte Instanzen öffentlich. Wenn jemand einen Link zu einem Beitrag oder Profil teilt, können Apps wie Discord, Slack oder iMessage eine Vorschau mit dem öffentlichen Beitrag, seinen Bildern und deinen öffentlichen Profilangaben abrufen. Beiträge mit Inhaltswarnung oder als sensibel markierte Beiträge erscheinen in der Vorschau ohne Text und Bilder. Lesezeichen, Blockierungen, Stummschaltungen, Meldungen und Einstellungen sind privat. Wenn du „Mich in Vorschlägen zeigen“ ausschaltest, kann man dich weiterhin über deinen genauen Nutzernamen finden.
docs_privacy_federation_h = 5. Föderation
docs_privacy_federation_p = Wenn du jemandem auf einer anderen tuita-Instanz folgst, antwortest oder schreibst, die Person erwähnst oder ihre Beiträge likest oder repostest, senden wir die dafür nötigen Daten an diese Instanz: dein öffentliches Profil, den Beitrag oder die Interaktion und bei Direktnachrichten den Nachrichtentext. Direktnachrichten an andere Instanzen sind nicht Ende-zu-Ende-verschlüsselt. Die Instanzen deiner Follower erhalten deine öffentlichen Beiträge und Profiländerungen. Diese Instanzen werden unabhängig betrieben und speichern Daten nach ihren eigenen Richtlinien. Wenn du einen Beitrag oder dein Konto löschst, bitten wir föderierte Instanzen, ihre Kopien zu löschen, können das aber nicht garantieren.
docs_privacy_cookies_h = 6. Cookies und lokaler Speicher
docs_privacy_cookies_p = Wir verwenden ein unbedingt erforderliches HttpOnly-Sitzungscookie. Deine Einstellungen für Design, Farbe, Textgröße und Bewegung werden im lokalen Speicher deines Browsers abgelegt und nie an uns gesendet.
docs_privacy_retention_h = 7. Wie lange wir Daten aufbewahren
docs_privacy_retention_1 = Inhalte und Kontodaten bleiben erhalten, bis du sie löschst.
docs_privacy_retention_2 = Gelöschte Beiträge verschwinden sofort aus der Öffentlichkeit, ihr Text und ihre Bilder werden gelöscht.
docs_privacy_retention_3 = Wenn du dein Konto löschst, entfernen wir deine Beiträge, Likes, Follows, Nachrichten und Sitzungen und anonymisieren den Kontoeintrag.
docs_privacy_retention_4 = Sitzungen laufen nach 30 Tagen ab oder sofort, wenn du dich abmeldest.
docs_privacy_retention_5 = Meldungen an die Moderation und das Admin-Protokoll werden so lange aufbewahrt, wie es für die Sicherheit des Dienstes nötig ist.
docs_privacy_rights_h = 8. Deine Rechte
docs_privacy_rights_p = Je nachdem, wo du lebst (zum Beispiel nach DSGVO oder CCPA), hast du das Recht auf Auskunft, Berichtigung, Export oder Löschung deiner Daten sowie auf Widerspruch gegen oder Einschränkung der Verarbeitung. Das meiste kannst du selbst unter Einstellungen → Konto erledigen: Profil bearbeiten, Daten herunterladen und Konto löschen. Für alles Weitere wende dich an:
docs_privacy_rights_end = Du kannst dich außerdem bei deiner zuständigen Datenschutzbehörde beschweren.
docs_privacy_security_h = 9. Sicherheit
docs_privacy_security_p = Passwörter werden mit scrypt gehasht. Sitzungstoken werden nur als Hashes gespeichert. Anfragen zwischen föderierten Instanzen werden mit Ed25519-Schlüsseln signiert. Kein System ist vollkommen sicher, verwende daher ein einzigartiges Passwort.
docs_privacy_children_h = 10. Kinder
docs_privacy_children_p = Der Dienst richtet sich nicht an Kinder unter dem in den Bedingungen genannten Mindestalter. Wenn du glaubst, dass ein Kind ein Konto hat, melde es.
docs_privacy_changes_h = 11. Änderungen
docs_privacy_changes_p = Änderungen dieser Erklärung veröffentlichen wir hier und aktualisieren das Datum oben. Wesentliche Änderungen kündigen wir in der App an.
docs_privacy_contact_h = 12. Kontakt
docs_privacy_contact_lead = Fragen oder Anliegen zum Datenschutz:
profile_posts =
    { $count ->
        [one] { $compact } Beitrag
       *[other] { $compact } Beiträge
    }
profile_edit = Profil bearbeiten
profile_more = Mehr
profile_copy_link = Profillink kopieren
profile_mute = { $handle } stummschalten
profile_unmute = Stummschaltung von { $handle } aufheben
profile_block = { $handle } blockieren
profile_unblock = Blockierung von { $handle } aufheben
profile_unblock_button = Blockierung aufheben
profile_suspended = Dieses Konto ist gesperrt. Nur das Team kann es sehen.
profile_muted = Stummgeschaltet
profile_org = Organisation
profile_role_admin = Admin
profile_role_moderator = Mod
profile_joined = Dabei seit { $date }
profile_following_label = Folge ich
profile_followers =
    { $count ->
        [one] Follower
       *[other] Follower
    }
profile_counts = Follow-Zahlen
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Gefolgt von { $a }
            }
        [2]
            { $others ->
               *[other] Gefolgt von { $a } und { $b }
            }
       *[other]
            { $others ->
                [one] Gefolgt von { $a }, { $b } und { $others } weiteren Person, der du folgst
               *[other] Gefolgt von { $a }, { $b } und { $others } weiteren Personen, denen du folgst
            }
    }
profile_banner = Profilbanner
profile_tabs = Profilbereiche
profile_tab_posts = Beiträge
profile_tab_replies = Antworten
profile_tab_media = Medien
profile_tab_likes = Likes
profile_posts_label = Beiträge
profile_empty_posts_self_title = Du hast noch nichts gepostet
profile_empty_posts_self_body = Deine Beiträge und Reposts erscheinen hier.
profile_empty_posts_title = { $handle } hat noch nichts gepostet
profile_empty_posts_body = Sobald es so weit ist, erscheinen die Beiträge hier.
profile_empty_replies_title = Noch keine Antworten
profile_empty_replies_body = Antworten erscheinen hier.
profile_empty_media_title = Noch keine Fotos
profile_empty_media_body = Beiträge mit Bildern erscheinen hier.
profile_empty_likes_title = Noch keine Likes
profile_empty_likes_body = Beiträge mit Like erscheinen hier.
profile_blocked_title = Du hast { $handle } blockiert
profile_blocked_body = Du siehst ihre Beiträge nicht, und sie können dir weder folgen noch schreiben.
profile_hidden_title = Du kannst diese Beiträge nicht sehen
profile_hidden_body = { $handle } hat eingeschränkt, wer das Profil sehen kann.
follows_tabs = Follow-Listen
follows_tab_followers = Follower
follows_tab_following = Folge ich
follows_label_followers = Follower
follows_label_following = Folge ich
follows_empty_followers_title = Noch keine Follower
follows_empty_followers_body = Wenn Leute diesem Konto folgen, erscheinen sie hier.
follows_empty_following_title = Folgt niemandem
follows_empty_following_body = Konten, denen diese Person folgt, erscheinen hier.
thread_title = Beitrag
thread_doc_title = { $name }: „{ $text }“
thread_views =
    { $count ->
        [one] Aufruf
       *[other] Aufrufe
    }
thread_reposts =
    { $count ->
        [one] Repost
       *[other] Reposts
    }
thread_quotes =
    { $count ->
        [one] Zitat
       *[other] Zitate
    }
thread_likes =
    { $count ->
        [one] Like
       *[other] Likes
    }
thread_stats = Beitragsstatistik
thread_replies = Antworten
thread_hidden = Ausgeblendete Antworten ({ $count })
thread_hidden_note = Die Autorin oder der Autor hat diese Antworten ausgeblendet.
thread_list_likes = Likes
thread_list_reposts = Repostet von
thread_list_quotes = Zitate
thread_list_empty = Noch niemand
tag_empty_title = Keine Beiträge mit { $tag }
tag_empty_body = Sei die erste Person, die ihn verwendet.
tag_posts_label = Beiträge mit { $tag }
bookmarks_title = Lesezeichen
bookmarks_label = Beiträge mit Lesezeichen
bookmarks_empty_title = Beiträge für später speichern
bookmarks_empty_body = Tippe bei einem Beitrag auf das Lesezeichen-Symbol, um ihn hier zu behalten. Nur du siehst deine Lesezeichen.
settings_account = Dein Konto
settings_account_desc = Profil und Passwort
settings_profile = Profil bearbeiten
settings_profile_desc = Name, Bio, Avatarform und Tinte
settings_privacy = Datenschutz & Sicherheit
settings_privacy_desc = Nachrichten, Auffindbarkeit, stummgeschaltete Wörter, blockierte und stummgeschaltete Konten
settings_notifications = Mitteilungen
settings_notifications_desc = Wähle, worüber du benachrichtigt wirst
settings_appearance = Darstellung
settings_appearance_desc = Design, Tinte, Textgröße, Kontrast und Bewegung
settings_language = Sprache
settings_language_desc = Sprache der Oberfläche
settings_language_note = In diesem Browser gespeichert und, wenn du angemeldet bist, in deinem Konto.
settings_about = Über diese Instanz
settings_about_desc = Regeln, Bedingungen, Datenschutz, Version und Tastenkürzel
settings_logout = Abmelden
settings_account_profile = Profil
settings_account_joined = Dabei seit { $date }
settings_account_edit = Profil bearbeiten
settings_password_title = Passwort ändern
settings_password_current = Aktuelles Passwort
settings_password_new = Neues Passwort
settings_password_hint = Mindestens 8 Zeichen. Andere Geräte werden abgemeldet.
settings_password_confirm = Neues Passwort bestätigen
settings_password_submit = Passwort aktualisieren
settings_password_mismatch = Die neuen Passwörter stimmen nicht überein.
settings_password_updated = Passwort aktualisiert. Andere Sitzungen wurden abgemeldet.
settings_sessions_title = Geräte & Sitzungen
settings_sessions_desc = Melde alles ab, was du nicht kennst, und ändere dann dein Passwort.
settings_device_on = { $browser } auf { $os }
settings_device_unknown = Unbekanntes Gerät
settings_session_current = Dieses Gerät
settings_session_last_active = Zuletzt aktiv: { $time }
settings_session_sign_out = Abmelden
settings_session_sign_out_device = { $device } abmelden
settings_session_signed_out = Sitzung abgemeldet
settings_sessions_revoke_all = Von allen anderen Geräten abmelden
settings_sessions_revoke_title = Andere Geräte abmelden?
settings_sessions_revoke_body = Alle Geräte außer diesem werden abgemeldet.
settings_sessions_revoke_confirm = Andere abmelden
settings_sessions_revoked =
    { $count ->
        [one] { $count } Sitzung abgemeldet
       *[other] { $count } Sitzungen abgemeldet
    }
settings_profile_preview = Vorschau
settings_profile_preview_alt = Avatar-Vorschau
settings_profile_uploads = Das Hochladen von Avatar und Banner folgt später. Bis dahin werden dein generierter Avatar und Banner verwendet.
settings_profile_details = Angaben
settings_profile_name = Anzeigename
settings_profile_bio = Bio
settings_profile_location = Ort
settings_profile_website = Website
settings_profile_shape = Avatarform
settings_profile_ink = Profiltinte
settings_profile_save = Profil speichern
settings_profile_saved = Profil aktualisiert
settings_shape_square = Quadrat
settings_shape_circle = Kreis
settings_shape_squircle = Squircle
settings_shape_octagon = Achteck
settings_ink_bone = Elfenbein
settings_ink_phosphor = Phosphor
settings_ink_amber = Bernstein
settings_ink_cyan = Cyan
settings_ink_rose = Rosé
settings_dm_title = Direktnachrichten
settings_dm_desc = Wer eine Unterhaltung mit dir beginnen kann
settings_dm_label = Wer dir schreiben kann
settings_dm_everyone = Alle
settings_dm_following = Leute, denen ich folge
settings_dm_none = Niemand
settings_dm_saved = Nachrichteneinstellungen gespeichert
settings_discover_title = Auffindbarkeit
settings_discoverable = Mich in Vorschlägen und Suche zeigen
settings_discoverable_hint = Man findet dich weiterhin über deinen genauen Nutzernamen
settings_show_client = Zeigen, von welchem Gerät ich poste
settings_show_client_hint = Fügt das Gerät unter deinen neuen Beiträgen hinzu
settings_words_title = Stummgeschaltete Wörter
settings_words_desc = Blende Beiträge mit diesen Wörtern, Sätzen oder #Hashtags in Timelines, Suche und Antworten aus. Deine eigenen Beiträge werden nie ausgeblendet.
settings_words_label = Wort zum Stummschalten
settings_words_placeholder = Wort, Satz oder #Hashtag hinzufügen
settings_words_add = Stummschalten
settings_words_remove = Stummschaltung von { $word } aufheben
settings_words_none = Keine stummgeschalteten Wörter.
settings_words_limit = Du kannst bis zu { $count } Wörter stummschalten.
settings_blocked_title = Blockierte Konten
settings_blocked_desc = Sie können dir nicht folgen, dir nicht schreiben und deine Beiträge weder sehen noch darauf reagieren. Sie werden nicht benachrichtigt.
settings_blocked_empty = Du hast niemanden blockiert.
settings_muted_title = Stummgeschaltete Konten
settings_muted_desc = Du siehst ihre Beiträge und Mitteilungen nicht. Sie können dir weiter folgen und schreiben.
settings_muted_empty = Du hast niemanden stummgeschaltet.
settings_unblock = Blockierung aufheben
settings_unmute = Stummschaltung aufheben
settings_unblock_user = Blockierung von { $handle } aufheben
settings_unmute_user = Stummschaltung von { $handle } aufheben
settings_notif_title = Benachrichtige mich über
settings_notif_desc = Stummgeschaltete und blockierte Konten benachrichtigen dich nie.
settings_notif_like = Likes
settings_notif_like_hint = Wenn jemandem dein Beitrag gefällt
settings_notif_repost = Reposts
settings_notif_repost_hint = Wenn jemand deinen Beitrag repostet
settings_notif_reply = Antworten
settings_notif_reply_hint = Wenn dir jemand antwortet
settings_notif_mention = Erwähnungen
settings_notif_mention_hint = Wenn dich jemand erwähnt
settings_notif_quote = Zitate
settings_notif_quote_hint = Wenn jemand deinen Beitrag zitiert
settings_notif_follow = Neue Follower
settings_notif_follow_hint = Wenn dir jemand folgt
settings_desktop_title = Desktop-Benachrichtigungen
settings_desktop_desc = Wird nur als Systembenachrichtigung angezeigt, solange dieser Tab im Hintergrund ist.
settings_desktop_state = Berechtigung: { $state }
settings_desktop_allowed = Erlaubt
settings_desktop_blocked = Blockiert
settings_desktop_not_asked = Nicht angefragt
settings_desktop_enable = Aktivieren
settings_desktop_blocked_toast = Benachrichtigungen sind für diese Seite in deinen Browsereinstellungen blockiert.
settings_desktop_unsupported = Dieser Browser unterstützt keine Desktop-Benachrichtigungen.
settings_theme_title = Design
settings_appearance_note = Auf diesem Gerät gespeichert und sofort angewendet.
settings_theme_mode = Design
settings_theme_system = System
settings_theme_light = Hell
settings_theme_dark = Dunkel
settings_theme_ink = Tinte
settings_theme_contrast = Kontrast
settings_contrast_normal = Normal
settings_contrast_more = Erhöht
settings_reading_title = Text und Bewegung
settings_theme_text_size = Textgröße
settings_theme_motion = Bewegung
settings_motion_system = System
settings_motion_reduce = Reduziert
settings_preview_title = Vorschau
settings_preview_text = So lesen sich Beiträge mit deinen Einstellungen.
settings_about_domain = Domain
settings_about_version = Version
settings_about_contact = Kontakt
settings_about_terms = Nutzungsbedingungen
settings_about_privacy = Datenschutzerklärung
settings_about_source = Quellcode
settings_about_rules = Regeln
explore_search_label = Beiträge, Leute und Tags suchen
explore_search_placeholder = Beiträge, Leute, #Tags suchen
explore_clear = Suche leeren
explore_tips = Suchtipps
explore_tips_title = Suchoperatoren
explore_op_from = Beiträge eines Kontos
explore_op_to = Antworten oder Erwähnungen an ein Konto
explore_op_media = Beiträge mit Bildern (oder has:poll, has:links)
explore_op_since = Am oder nach einem Datum gepostet
explore_op_until = Am oder vor einem Datum gepostet
explore_op_phrase = Wörter in dieser Reihenfolge
explore_op_exclude = Beiträge mit einem Wort ausschließen
explore_results = Suchergebnisse
explore_tab_posts = Beiträge
explore_tab_people = Leute
explore_tab_media = Medien
explore_people = Leute
explore_posts = Beiträge
explore_recent = Letzte Suchen
explore_recent_clear = Leeren
explore_trending = Im Trend
explore_trend_posts =
    { $count ->
        [one] { $compact } Beitrag
       *[other] { $compact } Beiträge
    }
explore_trends_empty = Noch keine Trends. Nutze einen #Hashtag!
explore_suggestions = Wem folgen
explore_suggestions_empty = Du folgst allen. Beeindruckend.
explore_no_posts_title = Keine Beiträge für „{ $q }“
explore_no_posts_body = Versuch es mit einem anderen Wort oder Hashtag.
explore_no_people_title = Keine Leute für „{ $q }“
explore_no_people_body = Versuch es mit einem Namen, einem Nutzernamen oder @name@instanz für jemanden auf einer anderen Instanz.
explore_no_media_title = Keine Bilder für „{ $q }“
explore_no_media_body = Passende Beiträge mit Bildern erscheinen hier.
notif_filter = Mitteilungsfilter
notif_tab_all = Alle
notif_tab_mentions = Erwähnungen
notif_empty_all_title = Noch keine Mitteilungen
notif_empty_all_body = Likes, Antworten, Reposts und neue Follower erscheinen hier.
notif_empty_mentions_title = Noch keine Erwähnungen
notif_empty_mentions_body = Antworten, Erwähnungen und Zitate deiner Beiträge erscheinen hier.
notif_who_two = { $first } und { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } und { $count } weitere Person
       *[other] { $first }, { $second } und { $count } weitere Personen
    }
notif_verb_like =
    { $count ->
        [one] gefällt dein Beitrag
       *[other] gefällt dein Beitrag
    }
notif_verb_repost =
    { $count ->
        [one] hat deinen Beitrag repostet
       *[other] haben deinen Beitrag repostet
    }
notif_verb_follow =
    { $count ->
        [one] folgt dir jetzt
       *[other] folgen dir jetzt
    }
notif_verb_follow_request =
    { $count ->
        [one] möchte dir folgen
       *[other] möchten dir folgen
    }
notif_verb_reply =
    { $count ->
        [one] hat dir geantwortet
       *[other] haben dir geantwortet
    }
notif_verb_mention =
    { $count ->
        [one] hat dich erwähnt
       *[other] haben dich erwähnt
    }
notif_verb_quote =
    { $count ->
        [one] hat deinen Beitrag zitiert
       *[other] haben deinen Beitrag zitiert
    }
notif_verb_poll =
    { $count ->
        [one] hat eine Umfrage beendet, an der du teilgenommen hast
       *[other] haben eine Umfrage beendet, an der du teilgenommen hast
    }
notif_verb_affiliation =
    { $count ->
        [one] lädt dich in die Organisation ein
       *[other] laden dich in die Organisation ein
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Planen
composer_post_all = Alle posten
composer_err_type = Diese Datei ist kein Bild oder Video, das tuita verwenden kann (PNG, JPEG, GIF, WebP, MP4, WebM oder MOV).
composer_err_image_size = Das Bild ist zu groß (max. 10 MB).
composer_err_video_size = Videos dürfen bis zu { $mb } MB groß sein.
composer_err_max_images = Du kannst bis zu 4 Bilder anhängen.
composer_err_exclusive = Ein Beitrag kann ein Video, Bilder oder eine Umfrage haben, aber nur eins davon.
composer_extra_placeholder = Weiteren Beitrag hinzufügen
composer_extra_label = Thread-Beitrag { $n }
composer_extra_remove = Thread-Beitrag { $n } entfernen
composer_schedule_chip = Wird { $time } gepostet
composer_schedule_clear = Nicht planen
composer_add_images = Bilder hinzufügen
composer_add_video = Video hinzufügen
composer_add_poll = Umfrage hinzufügen
composer_add_post = Weiteren Beitrag hinzufügen
composer_scheduled = Geplant für { $time }
composer_thread_sent = Thread mit { $count } Beiträgen gesendet
composer_remove_video = Video entfernen
composer_remove_image = Bild entfernen
composer_uploading = Wird hochgeladen
composer_uploading_video = Video wird hochgeladen … { $percent } %
composer_alt_video = Beschreibe dieses Video
composer_alt_image = Beschreibe dieses Bild
composer_alt_label = Beschreibung (Alt-Text) von Anhang { $index }
composer_emoji = Emoji hinzufügen
poll_length_hours = { $n } Std.
poll_length_days = { $n } T.
poll_choice = Option { $n }
poll_choice_optional = Option { $n } (optional)
poll_remove_choice = Option { $n } entfernen
poll_add_choice = Option hinzufügen
poll_length = Umfragedauer
poll_multiple_label = Mehrfachauswahl erlauben
poll_remove = Umfrage entfernen
schedule_title = Beitrag planen
schedule_hour = In einer Stunde
schedule_evening = Heute Abend
schedule_tomorrow = Morgen früh
schedule_week = In einer Woche
schedule_window = Wähle eine Zeit zwischen einer Minute und 90 Tagen ab jetzt.
schedule_custom = Datum und Uhrzeit wählen
schedule_set = Festlegen
schedule_clear = Nicht planen
schedule_view = Geplante Beiträge ansehen
dropzone_drop = Zum Anhängen ablegen
dropzone_reject = Diese Datei ist nicht nutzbar
nav_messages = Nachrichten
nav_lists = Listen
nav_scheduled = Geplant
notify_message = @{ $handle } hat dir eine Nachricht gesendet
shortcut_go_messages = Nachrichten
shortcut_go_lists = Listen
cropper_stage = Zuschneidebereich. Ziehe, um das Bild zu verschieben. Mit den Pfeiltasten verschiebst du es fein, mit Plus und Minus zoomst du.
cropper_hint = Zum Verschieben ziehen. Zum Zoomen zusammenziehen, scrollen oder den Regler nutzen.
cropper_zoom = Zoom
cropper_zoom_in = Vergrößern
cropper_zoom_out = Verkleinern
cropper_rotate = Drehen
cropper_reset = Zurücksetzen
cropper_apply = Übernehmen
image_field_avatar = Profilbild
image_field_banner = Banner
image_field_edit_avatar = Profilbild bearbeiten
image_field_edit_banner = Banner bearbeiten
image_field_change = Ändern
image_field_remove = Entfernen
image_field_hint = PNG, JPEG, GIF oder WebP bis 10 MB. Ein GIF wird zum Standbild.
image_field_not_image = Diese Datei ist kein Bild, das tuita verwenden kann (PNG, JPEG, GIF oder WebP).
image_field_too_large = Das Bild ist zu groß (max. 10 MB).
image_field_uploading = Wird hochgeladen
image_field_saved = Bild gespeichert.
image_field_removed = Bild entfernt.
edit_title = Beitrag bearbeiten
edit_note = Du kannst einen Beitrag 30 Minuten nach dem Posten bearbeiten. Man sieht, dass ein Beitrag bearbeitet wurde, und kann frühere Versionen einsehen.
edit_save = Speichern
edit_saved = Beitrag aktualisiert
edit_edited = Bearbeitet
edit_history_title = Bearbeitungsverlauf
edit_history_current = Aktuelle Version
edit_history_earlier = Frühere Version
edit_history_original = Original
report_post_title = Beitrag melden
report_user_title = { $handle } melden
report_intro = Was ist das Problem? Die Moderation dieser Instanz prüft deine Meldung. { $handle } sieht nicht, wer gemeldet hat.
report_reason = Grund
report_note = Noch etwas? (optional)
report_send = Meldung senden
report_sent = Danke. Deine Meldung wurde an das Moderationsteam geschickt.
report_duplicate = Das hast du schon gemeldet. Die Moderation schaut es sich an.
report_spam = Spam
report_spam_body = Unerwünschte Werbung, Betrug, sich wiederholende oder irreführende Inhalte
report_harassment = Belästigung
report_harassment_body = Jemanden angreifen, mobben oder bedrohen
report_hate = Hass
report_hate_body = Greift Menschen dafür an, wer sie sind
report_violence = Gewalt oder Selbstverletzung
report_violence_body = Drohungen, Gewaltverherrlichung, Aufforderung zur Selbstverletzung
report_misinformation = Falschinformation
report_misinformation_body = Falsche Behauptungen, die schaden könnten
report_impersonation = Identitätsbetrug
report_impersonation_body = Sich als jemand anderes ausgeben
report_other = Etwas anderes
report_other_body = Verstößt anders gegen die Regeln
post_edit = Beitrag bearbeiten
post_edit_history = Bearbeitungsverlauf
post_report = Beitrag melden
post_hide_reply = Antwort ausblenden
post_unhide_reply = Antwort einblenden
post_share_menu = Teilen …
post_reply_hidden = Ausgeblendete Antwort
post_reply_hidden_toast = Antwort ausgeblendet
post_reply_unhidden_toast = Antwort wieder sichtbar
sensitive_label = Sensible Medien
sensitive_show = Anzeigen
sensitive_hide = Sensible Medien ausblenden
media_label =
    { $count ->
        [one] { $count } Anhang
       *[other] { $count } Anhänge
    }
media_video = Video
media_open = Bild { $index } von { $count } öffnen
lightbox_title = Medienansicht
lightbox_prev = Zurück
lightbox_next = Weiter
poll_vote = Abstimmen
poll_your_vote = Deine Stimme
poll_votes =
    { $count ->
        [one] { $compact } Stimme
       *[other] { $compact } Stimmen
    }
poll_final = Endergebnis
poll_multiple = Mehrfachauswahl
poll_left_minutes = Noch { $n } Min.
poll_left_hours = Noch { $n } Std.
poll_left_days = Noch { $n } T.
pull_refresh = Zum Aktualisieren ziehen
messages_title = Nachrichten
messages_new = Neue Nachricht
messages_label = Unterhaltungen
messages_empty_title = Unterhaltung beginnen
messages_empty_body = Nachrichten, die du sendest und empfängst, erscheinen hier.
messages_you_prefix = Du: { $text }
messages_unread = { $count } ungelesen
messages_new_search = Leute suchen
messages_new_results = Leute
messages_new_none = Niemand gefunden.
chat_load_older = Frühere Nachrichten laden
chat_intro = Hier beginnt eure Unterhaltung.
chat_label = Nachrichten mit { $name }
chat_delete = Löschen
chat_seen = Gesehen
chat_delete_title = Diese Nachricht löschen?
chat_delete_body = Sie verschwindet für euch beide.
chat_doc_title = Chat mit { $name }
chat_error_title = Diese Unterhaltung konnte nicht geöffnet werden
chat_placeholder = Nachricht schreiben
chat_input_label = Nachricht
chat_send = Senden
lists_title = Listen
lists_new = Neue Liste
lists_edit = Liste bearbeiten
lists_label = Deine Listen
lists_empty_title = Noch keine Listen
lists_empty_body = Gruppiere Leute in Listen, um ihre Beiträge getrennt zu lesen.
lists_members =
    { $count ->
        [one] { $compact } Mitglied
       *[other] { $compact } Mitglieder
    }
lists_private = Privat
lists_name = Name
lists_description = Beschreibung
lists_private_switch = Private Liste
lists_save = Speichern
lists_picker_title = { $handle } zu Listen hinzufügen
lists_picker_none = Du hast noch keine Listen.
lists_delete = Liste löschen
lists_delete_title = Diese Liste löschen?
lists_delete_body = „{ $name }“ wird entfernt. Die Leute darin sind nicht betroffen.
lists_by = Liste von { $handle }
lists_tab_posts = Beiträge
lists_tab_members = Mitglieder
lists_tabs = Listenbereiche
lists_error_title = Diese Liste konnte nicht geöffnet werden
lists_posts_label = Listenbeiträge
lists_posts_empty_title = Hier ist noch nichts
lists_posts_empty_body = Beiträge der Mitglieder dieser Liste erscheinen hier.
lists_members_label = Listenmitglieder
lists_members_empty_title = Keine Mitglieder
lists_members_empty_body = Füge Leute über ihre Profile hinzu.
lists_remove = Entfernen
scheduled_title = Geplante Beiträge
scheduled_label = Geplante Beiträge
scheduled_status_pending = Geplant
scheduled_status_published = Gepostet
scheduled_status_failed = Fehlgeschlagen
scheduled_cw = Inhaltswarnung: { $text }
scheduled_poll = Umfrage mit { $count } Optionen, läuft { $hours } Std.
scheduled_change_time = Zeit ändern
scheduled_post_now = Jetzt posten
scheduled_remove = Entfernen
scheduled_window_error = Wähle eine Zeit zwischen einer Minute und 90 Tagen ab jetzt.
scheduled_when = Posten am
scheduled_zone = Zeitzone: { $zone }
scheduled_save = Speichern
scheduled_posted = Gepostet.
scheduled_cancel_title = Diesen geplanten Beitrag verwerfen?
scheduled_cancel_body = Er wird nicht gepostet.
scheduled_cancel_confirm = Beitrag verwerfen
scheduled_empty_title = Nichts geplant
scheduled_empty_body = Beiträge, die du planst, warten hier auf ihren Zeitpunkt.
profile_message = Nachricht
profile_add_to_lists = Zu Listen hinzufügen
media_alt = ALT
profile_report = { $handle } melden
chart_show_table = { $title } als Tabelle anzeigen
chart_col_day = Tag
chart_col_hour = Stunde (UTC)
chart_empty = { $title }: noch nichts anzuzeigen.
chart_bar_summary = { $title }: { $total } insgesamt, am meisten am { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } insgesamt, { $last } am { $label }.
chart_heat_summary = { $title }: { $total } insgesamt, am meisten los am { $label } um { $hour } ({ $peak }).
chart_heat_less = Weniger
chart_heat_more = Mehr
announce_region = Ankündigungen
announce_dismiss = Ankündigung schließen
announce_more = Mehr lesen
announce_tone_info = Hinweis
announce_tone_success = Gute Nachricht
announce_tone_warning = Warnung
admin_stat_people = Leute
admin_stat_people_sub = { $count } diese Woche aktiv
admin_stat_posts_today = Beiträge, 24 Std.
admin_stat_posts_total = { $count } insgesamt
admin_stat_reports = Offene Meldungen
admin_stat_reports_open = Muss geprüft werden
admin_stat_reports_clear = Alles erledigt
admin_stat_signups_today = Registrierungen heute
admin_stat_signups_sub = { $count } in 14 Tagen
admin_stat_suspended = Gesperrt
admin_stat_suspended_sub = Konten
admin_chart_posts = Beiträge pro Tag
admin_chart_signups = Registrierungen pro Tag
admin_chart_reports = Meldungen pro Tag
admin_chart_heat = Aktivität nach Wochentag und Uhrzeit
nav_admin = Verwaltung
admin_title = Verwaltung
admin_sections = Verwaltungsbereiche
admin_tab_overview = Übersicht
admin_tab_reports = Meldungen
admin_tab_users = Leute
admin_tab_instance = Instanz
admin_tab_federation = Föderation
admin_tab_announcements = Ankündigungen
admin_tab_invites = Einladungen
admin_tab_filter = Inhaltsfilter
admin_tab_audit = Protokoll
admin_denied_title = Nur für das Team
admin_denied_body = Dieser Bereich ist für die Administratoren und die Moderation dieser Instanz.
admin_denied_home = Zur Startseite
admin_unknown = unbekannt
admin_tag_active = Aktiv
admin_tag_suspended = Gesperrt
admin_tag_deleted = Gelöscht
admin_tag_org = Organisation
admin_audit_system = System
admin_audit_empty = Noch nichts protokolliert
admin_report_status = Meldungsstatus
admin_report_open = Offen
admin_report_resolved = Erledigt
admin_reports_empty_open = Keine offenen Meldungen
admin_reports_empty_open_body = Wenn Leute Beiträge oder Konten melden, erscheinen sie hier.
admin_reports_empty_resolved = Noch nichts erledigt
admin_report_by = gemeldet von { $handle }
admin_report_same_target = { $count } offene Meldungen
admin_report_media =
    { $count ->
        [one] { $count } Anhang
       *[other] { $count } Anhänge
    }
admin_report_poll = Enthält eine Umfrage
admin_report_dismiss = Abweisen
admin_report_delete_post = Beitrag löschen
admin_report_suspend = Konto sperren
admin_report_delete_suspend = Löschen und sperren
admin_report_confirm_title = Maßnahme ausführen?
admin_report_confirm_body = { $action }. Das wird im Protokoll festgehalten.
admin_report_confirm = Bestätigen
admin_report_resolution = { $action } von { $handle } · { $time }
admin_res_dismiss = Abgewiesen
admin_res_delete_post = Beitrag gelöscht
admin_res_suspend_user = Konto gesperrt
admin_res_delete_and_suspend = Beitrag gelöscht und Konto gesperrt
admin_users_search = Leute suchen
admin_users_search_placeholder = Name oder Nutzername
admin_users_show = Anzeigen
admin_users_empty = Keine Leute gefunden
admin_filter_local = Lokal
admin_filter_remote = Extern
admin_filter_staff = Team
admin_filter_verified = Verifiziert
admin_filter_suspended = Gesperrt
admin_filter_all = Alle
admin_user_details = Details zu { $handle }
admin_user_menu = { $handle } verwalten
admin_user_posts =
    { $count ->
        [one] { $compact } Beitrag
       *[other] { $compact } Beiträge
    }
admin_user_followers =
    { $count ->
        [one] { $compact } Follower
       *[other] { $compact } Follower
    }
admin_user_joined = beigetreten { $time }
admin_user_active = aktiv { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } offene Meldung
       *[other] { $compact } offene Meldungen
    }
admin_act_verify = Verifizieren
admin_act_unverify = Verifizierung entfernen
admin_act_org = Zur Organisation machen
admin_act_unorg = Organisationsstatus entfernen
admin_act_role_user = Teamrolle entfernen
admin_act_role_moderator = Zum Moderator machen
admin_act_role_admin = Zum Admin machen
admin_act_suspend = Sperren
admin_act_unsuspend = Entsperren
admin_act_purge = Alle Beiträge löschen
admin_act_rename = Nutzernamen ändern
admin_act_reset_name = Anzeigenamen zurücksetzen
admin_act_clear_bio = Bio leeren
admin_act_delete = Konto löschen
admin_done_verified = { $handle } ist verifiziert
admin_done_unverified = Verifizierung entfernt
admin_done_org = { $handle } ist jetzt eine Organisation
admin_done_unorg = Organisationsstatus entfernt
admin_done_role = Rolle aktualisiert
admin_done_suspended = { $handle } gesperrt
admin_done_unsuspended = { $handle } entsperrt
admin_done_purged =
    { $count ->
        [one] { $count } Beitrag gelöscht
       *[other] { $count } Beiträge gelöscht
    }
admin_done_name_reset = Anzeigename zurückgesetzt
admin_done_bio_cleared = Bio geleert
admin_done_deleted = { $handle } gelöscht
admin_done_renamed = Jetzt { $handle }
admin_role_phrase_user = Nutzer
admin_role_phrase_moderator = Moderator
admin_role_phrase_admin = Admin
admin_role_title = Rolle von { $handle } ändern?
admin_role_body = Die Person erhält die Rolle „{ $role }“.
admin_role_confirm = Rolle ändern
admin_unorg_title = Organisationsstatus von { $handle } entfernen?
admin_unorg_body = Alle Zugehörigkeiten enden.
admin_unorg_confirm = Entfernen
admin_suspend_title = { $handle } sperren?
admin_suspend_body = Die Person wird abgemeldet und ihre Inhalte bleiben ausgeblendet, bis du sie entsperrst.
admin_suspend_confirm = Sperren
admin_purge_title = Jeden Beitrag von { $handle } löschen?
admin_purge_body = Das lässt sich nicht rückgängig machen.
admin_purge_confirm = Alle löschen
admin_reset_name_title = Anzeigenamen von { $handle } zurücksetzen?
admin_reset_name_body = Er wird durch den Nutzernamen ersetzt.
admin_reset_name_confirm = Zurücksetzen
admin_delete_title = { $handle } löschen?
admin_delete_body = Beiträge, Follows und Nachrichten werden entfernt und das Konto wird anonymisiert. Das lässt sich nicht rückgängig machen.
admin_delete_confirm = Konto löschen
admin_rename_title = Nutzernamen von { $handle } ändern
admin_rename_note = Links zum alten Nutzernamen funktionieren nicht mehr.
admin_rename_handle = Neuer Nutzername
admin_rename_reset_name = Anzeigenamen ebenfalls zurücksetzen
admin_rename_save = Nutzernamen ändern
admin_detail_title = Kontodetails
admin_detail_invite = Einladung { $code }
admin_detail_posts = Beiträge
admin_detail_followers = Follower
admin_detail_following = Folge ich
admin_detail_open_reports = Offene Meldungen
admin_detail_sessions = Sitzungen
admin_detail_passkeys = Passkeys
admin_detail_tokens = API-Tokens
admin_detail_joined = Dabei seit { $time }
admin_detail_recent_posts = Neueste Beiträge
admin_detail_no_posts = Keine Beiträge.
admin_detail_reports = Meldungen zu diesem Konto
admin_detail_no_reports = Keine Meldungen.
admin_detail_history = Moderationsverlauf
admin_detail_nothing = Noch nichts.
admin_instance_title = Instanz
admin_instance_name = Name der Instanz
admin_instance_description = Beschreibung
admin_instance_contact = Kontakt-E-Mail
admin_instance_contact_hint = Wird auf den Seiten Bedingungen und Datenschutz angezeigt
admin_instance_min_age = Mindestalter
admin_instance_rules = Regeln
admin_instance_rules_hint = Eine Regel pro Zeile
admin_instance_preview = Bedingungen ansehen
admin_instance_save = Speichern
admin_instance_saved = Instanzeinstellungen gespeichert
admin_instance_template_note = Die Nutzungsbedingungen und die Datenschutzerklärung sind Vorlagen, die diese Werte verwenden. Lass sie für deine Rechtsordnung prüfen, bevor du die Instanz öffentlich öffnest.
admin_signups_title = Registrierung
admin_signups_desc = Wer auf dieser Instanz ein Konto erstellen darf
admin_signups_open = Offen
admin_signups_open_hint = Jeder kann ein Konto erstellen.
admin_signups_invite = Nur mit Einladung
admin_signups_invite_hint = Man braucht einen Einladungscode aus dem Tab Einladungen.
admin_signups_closed = Geschlossen
admin_signups_closed_hint = Niemand kann sich registrieren.
admin_filter_title = Inhaltsfilter
admin_filter_desc = Nichts wird blockiert, bis du unten Wörter hinzufügst. Blockierte Wörter gelten für Nutzernamen, Anzeigenamen, Orte, Bios, Beiträge und Listennamen, auch bei verschleierten Schreibweisen. Reservierte Nutzernamen wie „admin“ können nicht registriert werden.
admin_filter_blocked = Blockierte Wörter
admin_filter_blocked_hint = Eines pro Zeile, auch innerhalb von Wörtern
admin_filter_allowed = Erlaubte Wörter
admin_filter_allowed_hint = Ausnahmen zu den blockierten Wörtern, eines pro Zeile
admin_filter_reserved = Weitere reservierte Nutzernamen
admin_filter_reserved_hint = Einer pro Zeile
admin_filter_save = Filter speichern
admin_filter_saved = Filter gespeichert
admin_filter_field_handle = Nutzername
admin_filter_field_name = Name
admin_filter_field_location = Ort
admin_filter_field_bio = Bio
admin_scan_title = Konten prüfen
admin_scan_desc = Finde Konten, deren Nutzername, Name oder Bio gegen den Filter verstoßen.
admin_scan_run = Jetzt prüfen
admin_scan_clean = Kein Konto verstößt gegen den Filter.
admin_scan_found =
    { $count ->
        [one] { $count } Konto braucht Aufmerksamkeit
       *[other] { $count } Konten brauchen Aufmerksamkeit
    }
admin_scan_reason = Feld „{ $field }“ enthält { $reason }
admin_ann_title = Neue Ankündigung
admin_ann_desc = Ein Banner am oberen Rand des Bildschirms aller, bis es abläuft oder du es beendest. Man kann es schließen.
admin_ann_text = Nachricht
admin_ann_link = Link
admin_ann_link_hint = Optional: https://… oder /pfad
admin_ann_tone = Stil
admin_ann_tone_info = Info
admin_ann_tone_success = Gute Nachricht
admin_ann_tone_warning = Warnung
admin_ann_duration = Dauer
admin_ann_days =
    { $count ->
        [one] { $count } Tag
       *[other] { $count } Tage
    }
admin_ann_week = 1 Woche
admin_ann_until_ended = Bis zum Beenden
admin_ann_publish = Veröffentlichen
admin_ann_published = Ankündigung veröffentlicht
admin_ann_list = Bisherige Ankündigungen
admin_ann_none = Noch keine Ankündigungen.
admin_ann_end = Beenden
admin_ann_ended = beendet
admin_ann_expired = abgelaufen
admin_ann_inactive = Nicht sichtbar
admin_ann_until = bis { $date }
admin_inv_title = Einladungscodes
admin_inv_desc = Lass bestimmte Leute beitreten, solange die Registrierung nur mit Einladung möglich ist.
admin_inv_note = Notiz (für wen)
admin_inv_uses = Verwendungen
admin_inv_use_count =
    { $count ->
        [one] { $count } Verwendung
       *[other] { $count } Verwendungen
    }
admin_inv_expiry = Läuft ab
admin_inv_week = In 1 Woche
admin_inv_days =
    { $count ->
        [one] In { $count } Tag
       *[other] In { $count } Tagen
    }
admin_inv_never = Nie
admin_inv_create = Code erstellen
admin_inv_created = Code { $code } erstellt
admin_inv_created_copied = Code { $code } erstellt und kopiert
admin_inv_list = Codes
admin_inv_none = Noch keine Einladungscodes.
admin_inv_state_active = Aktiv
admin_inv_state_revoked = Widerrufen
admin_inv_state_used = Aufgebraucht
admin_inv_state_expired = Abgelaufen
admin_inv_used = { $uses } von { $max } verwendet
admin_inv_expires = läuft am { $date } ab
admin_inv_copy = Code { $code } kopieren
admin_inv_copied = Code kopiert
admin_inv_copy_failed = Kopieren fehlgeschlagen. Der Code lautet { $code }.
admin_inv_revoke = Widerrufen
admin_inv_revoke_title = Diese Einladung widerrufen?
admin_inv_revoke_body = Wer damit schon beigetreten ist, behält sein Konto.
settings_devices = Geräte & Sitzungen
settings_devices_desc = Sieh, wo du angemeldet bist, und melde dich aus der Ferne ab
settings_passkeys = Passkeys
settings_passkeys_desc = Melde dich mit Fingerabdruck, Gesicht, Bildschirmsperre oder Sicherheitsschlüssel an
settings_affiliation = Zugehörigkeit
settings_affiliation_desc = Organisationen, zu denen du gehörst, oder die Konten deiner Organisation
settings_bots = Bots & API
settings_bots_desc = API-Tokens und das Label „Automatisiert“
settings_saved = Gespeichert
passkey_title = Deine Passkeys
passkey_desc = Melde dich mit Fingerabdruck, Gesicht, Bildschirmsperre oder Sicherheitsschlüssel statt mit einem Passwort an. Passkeys lassen sich nicht phishen, und auf unserem Server wird nichts Geheimes gespeichert.
passkey_none = Noch keine Passkeys.
passkey_add = Passkey hinzufügen
passkey_added = Passkey „{ $name }“ hinzugefügt
passkey_default_name = Passkey
passkey_added_ago = hinzugefügt { $time }
passkey_used_ago = zuletzt verwendet { $time }
passkey_never_used = nie verwendet
passkey_rename = Umbenennen
passkey_rename_label = Passkey { $name } umbenennen
passkey_rename_title = Passkey umbenennen
passkey_name = Name
passkey_rename_save = Speichern
passkey_remove = Entfernen
passkey_remove_label = Passkey { $name } entfernen
passkey_remove_title = „{ $name }“ entfernen?
passkey_remove_body = Du kannst dich mit diesem Passkey nicht mehr anmelden. Vielleicht möchtest du ihn auch von deinem Gerät oder aus deinem Passwortmanager löschen.
passkey_removed = Passkey entfernt
passkey_unsupported = Dieser Browser kann Passkeys unter dieser Adresse nicht verwenden.
passkey_needs_domain = Für Passkeys braucht diese Seite einen Domainnamen, keine IP-Adresse.
passkey_already_here = Dieses Gerät hat bereits einen Passkey für dieses Konto.
token_scope_read = Lesen
token_scope_write = Posten und interagieren
token_scope_messages = Direktnachrichten
token_scope_read_hint = Timelines, Mitteilungen, Profile, Suche und der Echtzeit-Stream
token_scope_write_hint = Posten, Löschen, Likes, Reposts, Lesezeichen, Follows und Uploads
token_scope_messages_hint = Direktnachrichten lesen und senden
token_title = API-Tokens
token_desc = Mit Tokens können Skripte und Bots dein Konto über die API nutzen. Behandle sie wie Passwörter: Wer einen Token hat, kann im Rahmen seiner Rechte als du handeln.
token_none = Noch keine Tokens.
token_name = Name des Tokens
token_name_hint = Zum Beispiel: Wetter-Bot
token_scopes = Berechtigungen
token_create = Token erstellen
token_used_ago = zuletzt verwendet { $time }
token_never_used = nie verwendet
token_revoke = Widerrufen
token_revoke_label = Token { $name } widerrufen
token_revoke_title = „{ $name }“ widerrufen?
token_revoke_body = Alles, was ihn verwendet, hört auf zu funktionieren.
token_revoked = Token widerrufen
token_secret_title = Dein neuer Token
token_secret_warning = Kopiere ihn jetzt. Aus Sicherheitsgründen wird er nicht noch einmal angezeigt.
token_secret_label = Neuer API-Token
token_copy = Kopieren
token_copied = Token kopiert
token_copy_failed = Kopieren fehlgeschlagen. Markiere den Token und kopiere ihn von Hand.
token_done = Fertig
bot_title = Automatisiertes Konto
bot_desc = Zeigt ein Label „Automatisiert“ neben dem Namen, damit alle wissen, dass ein Bot postet.
bot_switch = Dies ist ein automatisiertes Konto
bot_owner = Betrieben von
bot_owner_hint = Der Nutzername der Person oder Organisation dahinter. Optional.
bot_owner_save = Speichern
bot_docs = Entwicklerdoku
bot_docs_desc = Endpunkte, Echtzeit-Events und Beispielbots
aff_yours_title = Deine Zugehörigkeit
aff_yours_desc = Eine Organisation kann dich einladen, ihr Abzeichen neben deinem Namen zu zeigen. Nichts ändert sich, bis du annimmst, und du kannst jederzeit austreten.
aff_none = Du gehörst zu keiner Organisation.
aff_invites_title = Einladungen
aff_accept = Annehmen
aff_decline = Ablehnen
aff_leave = Verlassen
aff_leave_title = Diese Organisation verlassen?
aff_leave_body = Ihr Abzeichen verschwindet von deinem Profil.
aff_remove_title = { $handle } entfernen?
aff_remove_body = Dein Abzeichen verschwindet von ihrem Profil.
aff_remove = Entfernen
aff_remove_label = { $handle } entfernen
aff_ended = Zugehörigkeit beendet
aff_declined = Einladung abgelehnt
aff_accepted = Zugehörigkeit hinzugefügt
aff_invited = Einladung gesendet
aff_org_title = Zugehörige Konten
aff_org_desc = Lade Leute ein, dein Abzeichen neben ihrem Namen zu zeigen. Sie müssen annehmen.
aff_handle = Nutzername
aff_label = Bezeichnung
aff_label_hint = Optional, zum Beispiel „Entwickler“
aff_invite = Einladung senden
aff_members_none = Noch keine zugehörigen Konten.
aff_pending = Eingeladen
aff_not_org = Bist du eine Organisation? Bitte ein Admin dieser Instanz, dein Konto so zu markieren.
auth_passkey = Mit Passkey anmelden
auth_or = oder
auth_invite = Einladungscode
auth_invite_hint = Diese Instanz nimmt neue Leute nur auf Einladung auf.
auth_invite_required = Gib den Einladungscode ein, den du erhalten hast.
landing_developers = Entwickler
landing_invite = Die Registrierung ist nur mit Einladung möglich. Du hast einen Code? Erstelle dein Konto.
dev_title = Entwickler
dev_hero = Bots & API
dev_hero_sub = Erstelle Bots, Skripte und Integrationen für { $name }
dev_intro_h = Entwickle für { $name }
dev_intro_p = Alles, was die tuita-Apps tun, läuft über eine kleine JSON-API, die auch Bots und Skripte nutzen können. Du brauchst ein Konto auf { $name } und einen persönlichen API-Token. Eine App-Registrierung oder einen OAuth-Ablauf gibt es nicht.
dev_intro_1 = Erstelle ein Konto für deinen Bot oder verwende dein eigenes.
dev_intro_2 = Aktiviere unter Einstellungen, Bots & API die Option „Dies ist ein automatisiertes Konto“, damit man ein Label „Automatisiert“ sieht, und gib an, wer es betreibt.
dev_intro_3 = Erstelle einen Token mit den Berechtigungen, die du brauchst, und halte ihn geheim.
dev_auth_h = Authentifizierung
dev_auth_p = Sende den Token im Authorization-Header. Jede Anfrage und Antwort ist JSON (UTF-8), außer Uploads, die multipart sind.
dev_auth_prefix = Ein Token ist tuita_ gefolgt von 43 Buchstaben, Ziffern, Bindestrichen und Unterstrichen. Es wird nur ein Hash gespeichert, ein verlorener Token kann daher nicht erneut angezeigt werden: Erstelle einen neuen. Ein Token hat eine oder mehrere dieser Berechtigungen:
dev_auth_refused = Ein Token kann keine Passwörter, Sitzungen, Tokens oder Passkeys ändern, das Konto weder exportieren noch löschen und die Admin-API nicht nutzen. Dafür ist eine echte Anmeldung auf der Website nötig, sonst lautet die Antwort 403.
dev_auth_errors = Fehler kommen als JSON mit einer Fehlermeldung und einem 4xx-Status zurück. 401 bedeutet, dass der Token falsch oder widerrufen ist, 403 eine fehlende Berechtigung oder eine abgelehnte Route, und 429 kommt mit einem Retry-After-Header.
dev_post_h = Dein erster Beitrag
dev_post_p = Lade Bilder und Videos zuerst mit POST /api/v1/media (multipart) hoch und hänge sie dann per ID in mediaIds an. Eine Antwort setzt replyTo, ein Zitat setzt quoteOf.
dev_endpoints_h = Endpunkte
dev_endpoints_intro = Pfade in geschweiften Klammern sind Platzhalter. Listen werden mit ?before= seitenweise abgerufen, wobei der nächste Wert der vorherigen Seite gesetzt wird; limit ist 1 bis 40. Die Spalte „Berechtigung“ nennt die Berechtigung, die ein Token braucht.
dev_col_method = Methode
dev_col_path = Pfad
dev_col_scope = Berechtigung
dev_col_what = Was es tut
dev_ep_me = Das Konto, zu dem der Token gehört
dev_ep_patch_me = Profil und Einstellungen dieses Kontos ändern
dev_ep_bot = Das Label „Automatisiert“ ein- oder ausschalten und die Person nennen, die den Bot betreibt
dev_ep_following = Die Start-Timeline der Leute, denen du folgst
dev_ep_foryou = Die Timeline „Für dich“; seitenweise mit ?cursor=
dev_ep_notifications = Mitteilungen (?filter=mentions für Antworten, Erwähnungen und Zitate)
dev_ep_post = Ein Beitrag mit Thread und Antworten
dev_ep_user = Ein Profil (handle@domain für Konten auf anderen Servern)
dev_ep_user_posts = Beiträge einer Person (?tab=replies, media oder likes; bei media auch ?kind=photo oder video)
dev_ep_search = Beiträge, Leute oder Medien suchen, mit Operatoren wie from: und has:media
dev_ep_trends = Die in der letzten Woche am meisten verwendeten Tags
dev_ep_stream = Echtzeit-Ereignisse (Server-Sent Events)
dev_ep_create = Einen Beitrag schreiben
dev_ep_thread = 2 bis 25 Beiträge als Thread schreiben
dev_ep_delete = Einen deiner Beiträge löschen
dev_ep_like = Einen Beitrag liken (DELETE zum Zurücknehmen). Auch /repost und /bookmark
dev_ep_follow = Jemandem folgen (DELETE zum Entfolgen)
dev_ep_media = Ein Bild oder Video als multipart hochladen; die Antwort enthält die ID zum Anhängen
dev_ep_schedule = Einen Beitrag für später planen
dev_ep_report = Einen Beitrag oder ein Konto der Moderation melden
dev_ep_conversations = Deine Direktnachrichten-Unterhaltungen
dev_ep_message = Eine Direktnachricht senden; der Body ist JSON mit einem Feld text
dev_realtime_h = Echtzeit-Ereignisse
dev_realtime_p = GET /api/v1/stream ist ein Server-Sent-Events-Stream. Er sendet ein notification-Ereignis für jedes Like, jeden Repost, Follow, jede Antwort, Erwähnung oder jedes Zitat, ein message-Ereignis für Direktnachrichten und badges mit Zahlen ungelesener Einträge. Alle 25 Sekunden sendet er eine Kommentarzeile und bittet Clients, es nach 5 Sekunden erneut zu versuchen; verbinde dich neu, wenn die Verbindung abbricht.
dev_example_h = Beispiel: ein Bot, der auf Erwähnungen antwortet
dev_example_node = Node 22 oder neuer, keine Pakete:
dev_example_python = Dasselbe in Python, nur mit requests:
dev_limits_h = Limits
dev_limit_rate = Jeder Token darf 600 Anfragen pro 10 Minuten stellen. Bei Überschreitung antwortet der Server mit 429 und einem Retry-After-Header.
dev_limit_tokens = Ein Konto kann bis zu 20 Tokens haben und bis zu 20 Tokens pro Stunde erstellen.
dev_limit_posting = Posten, Folgen, Melden und die anderen Aktionen haben eigene Limits pro Konto.
dev_rules_h = Regeln für Bots
dev_rule_1 = Aktiviere das Label „Automatisiert“ und gib an, wer den Bot betreibt.
dev_rule_2 = Antworte oder schreibe nur Leuten, die es wollten: Sie haben den Bot erwähnt, ihm gefolgt oder ihm zuerst geschrieben. Sende keine ungebetenen Erwähnungen oder Direktnachrichten.
dev_rule_3 = Mach langsam: Bleib deutlich unter den obigen Limits.
dev_rule_4 = Beachte die Bedingungen und Regeln der Instanz. Die Moderation kann Bots wie jedes Konto sperren.
admin_act_log_admin_bootstrap = Erster Admin eingerichtet
admin_act_log_announcement_create = Ankündigung veröffentlicht
admin_act_log_announcement_end = Ankündigung beendet
admin_act_log_instance_settings = Instanzeinstellungen geändert
admin_act_log_invite_create = Einladung erstellt
admin_act_log_invite_revoke = Einladung widerrufen
admin_act_log_post_delete = Beitrag gelöscht
admin_act_log_report_dismiss = Meldung abgewiesen
admin_act_log_user_delete = Konto gelöscht
admin_act_log_user_org = Als Organisation markiert
admin_act_log_user_unorg = Organisationsstatus entfernt
admin_act_log_user_purge_posts = Alle Beiträge gelöscht
admin_act_log_user_rename = Konto umbenannt
admin_act_log_user_role = Rolle geändert
admin_act_log_user_suspend = Konto gesperrt
admin_act_log_user_unsuspend = Konto entsperrt
admin_act_log_user_unverify = Verifizierung entfernt
admin_act_log_user_verify = Konto verifiziert
admin_act_log_domain_block = Instanz blockiert
admin_act_log_domain_unblock = Instanz entblockt
admin_act_log_federation_retry = Neuer Zustellversuch angefordert
admin_fed_stat_domains = Bekannte Instanzen
admin_fed_stat_domains_sub = von dieser Instanz gesehen
admin_fed_stat_accounts = Externe Konten
admin_fed_stat_accounts_sub = von anderswo bekannt
admin_fed_stat_posts = Externe Beiträge
admin_fed_stat_posts_sub = lokal gespeichert
admin_fed_stat_queue = In der Warteschlange
admin_fed_stat_queue_sub = wartende Zustellungen
admin_fed_queue_title = Zustellwarteschlange
admin_fed_queue_desc = Signierte Aktivitäten, die darauf warten, andere Instanzen zu erreichen. Fehlgeschlagene werden mit wachsenden Abständen erneut versucht und nach 12 Versuchen verworfen.
admin_fed_queue_empty = Nichts wartet. Jede Zustellung wurde gesendet.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } Zustellung wartet
       *[other] { $compact } Zustellungen warten
    }
admin_fed_queue_due = { $count } jetzt fällig
admin_fed_queue_oldest = Älteste wartet seit { $date }
admin_fed_retry = Jetzt erneut versuchen
admin_fed_retry_done =
    { $count ->
        [one] { $compact } Zustellung wird jetzt erneut versucht.
       *[other] { $compact } Zustellungen werden jetzt erneut versucht.
    }
admin_fed_failing = Fehlerhafte Instanzen
admin_fed_failing_none = Keine Instanz hat Fehler.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } Zustellung
       *[other] { $compact } Zustellungen
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } Versuch
       *[other] { $compact } Versuche
    }
admin_fed_failing_next = nächster Versuch { $date }
admin_fed_failing_error = Letzter Fehler
admin_fed_failing_no_error = Kein Fehler aufgezeichnet
admin_fed_actor_title = Instanz-Akteur
admin_fed_actor_desc = Andere Instanzen prüfen die Anfragen, die diese Instanz signiert, anhand dieses Schlüssels.
admin_fed_actor_id = Akteur
admin_fed_actor_key = Schlüssel-ID
admin_fed_actor_fingerprint = SHA-256-Fingerabdruck
admin_fed_actor_copy = Fingerabdruck kopieren
admin_fed_actor_copied = Fingerabdruck kopiert
admin_fed_actor_copy_failed = Kopieren fehlgeschlagen. Der Fingerabdruck lautet { $fingerprint }
admin_fed_blocks_title = Blockierte Instanzen
admin_fed_blocks_desc = Stummschalten blendet die Konten einer Instanz in öffentlichen Timelines und der Suche aus. Sperren blendet sie überall aus, verwirft, was sie senden, und stoppt Zustellungen an sie.
admin_fed_domain = Domain
admin_fed_domain_hint = example.social oder ein Link zu einer beliebigen Seite darauf
admin_fed_severity = Schweregrad
admin_fed_silence = Stummschalten
admin_fed_suspend = Sperren
admin_fed_reason = Grund
admin_fed_reason_hint = Optional, nur für das Team sichtbar
admin_fed_block = Instanz blockieren
admin_fed_blocked = { $domain } ist blockiert
admin_fed_none = Keine Instanzen blockiert.
admin_fed_list = Blockierungen
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } bekanntes Konto
       *[other] { $compact } bekannte Konten
    }
admin_fed_row_no_reason = Kein Grund angegeben
admin_fed_row_changed = { $domain } ist jetzt auf { $severity } gesetzt
admin_fed_remove = Entfernen
admin_fed_remove_label = Blockierung von { $domain } aufheben
admin_fed_remove_title = Blockierung von { $domain } aufheben?
admin_fed_remove_body = Konten und Beiträge kommen zurück und Zustellungen beginnen wieder.
admin_fed_removed = { $domain } ist nicht mehr blockiert
remote_domain_title = Konto auf { $domain }
remote_view_original = Auf { $domain } ansehen
visibility_unlisted = Nicht gelistet
visibility_unlisted_hint = Wird nicht in öffentlichen Timelines gezeigt
visibility_followers = Nur Follower
visibility_followers_hint = Nur Follower können das sehen
visibility_direct = Direkt
visibility_direct_hint = Nur die erwähnten Leute können das sehen
follow_requested = Angefragt
follow_cancel_request = Folgeanfrage zurückziehen
follow_requested_toast = Folgeanfrage an { $handle } gesendet
follow_cancelled_toast = Folgeanfrage zurückgezogen
profile_locked = Genehmigt Follower
profile_locked_hint = Folgeanfragen müssen von diesem Konto genehmigt werden.
profile_original = Profil auf { $domain } ansehen
profile_remote_note = Eine Kopie von { $domain }. Zahlen und Beiträge entsprechen dem, was diese Instanz gesehen hat.
remote_lookup_loading = { $handle } auf { $domain } wird gesucht …
remote_lookup_notfound_title = Kein solches Konto
remote_lookup_notfound_body = { $handle } existiert nicht oder { $domain } gibt es nicht frei.
remote_lookup_failed_title = { $handle } konnte nicht geladen werden
explore_lookup = { $target } wird gesucht …
explore_lookup_empty_body = Dieses Konto oder dieser Beitrag wurde nicht gefunden. Die andere Instanz ist vielleicht offline oder gibt ihn nicht frei.
language_label = Sprache
oauth_title = App autorisieren
oauth_loading = Anfrage wird geprüft …
oauth_wants = Diese App möchte dein Konto { $handle } verwenden.
oauth_name_warning = Jeder kann einen beliebigen App-Namen wählen. Genehmige nur Apps, denen du vertraust.
oauth_website = Website
oauth_allow = Sie kann
oauth_scope_read = Dein Profil, Beiträge, Follows, Listen, Timelines und Mitteilungen sowie nicht Ende-zu-Ende-verschlüsselte Direktnachrichten lesen.
oauth_scope_write = Für dich posten, bearbeiten und löschen, liken, folgen, blockieren, stummschalten sowie dein Profil und deine Einstellungen ändern.
oauth_scope_messages = Deine Direktnachrichten lesen und senden.
oauth_push_note = Push-Benachrichtigungen werden nicht unterstützt. Eine App, die sie anfordert, erhält sie nicht.
oauth_redirect = Nach deiner Antwort wirst du weitergeleitet zu
oauth_redirect_oob = Die App erhält keine Weiterleitung. Hier wird ein Code angezeigt, den du in die App einfügst.
oauth_approve = Autorisieren
oauth_deny = Ablehnen
oauth_switch = Anderes Konto verwenden
oauth_sent_approved = Autorisiert. Du wirst zu { $host } zurückgeleitet. Falls nichts passiert, schließe diesen Tab.
oauth_sent_denied = Abgelehnt. Du wirst zu { $host } zurückgeleitet. Falls nichts passiert, schließe diesen Tab.
oauth_denied = Abgelehnt. Nichts wurde mit der App geteilt.
oauth_code_title = Autorisierungscode
oauth_code_body = Füge diesen Code in die App ein. Er funktioniert einmal und läuft nach 10 Minuten ab.
oauth_code_copy = Code kopieren
oauth_code_copied = Code kopiert
oauth_code_copy_failed = Kopieren fehlgeschlagen. Markiere den Code und kopiere ihn von Hand.
oauth_error_title = Diese Anfrage kann nicht autorisiert werden
oauth_error_note = Es wurde nichts an die App gesendet.
oauth_error_invalid_request = Die Anfrage ist fehlerhaft oder will zu einer Adresse zurückkehren, die die App nicht registriert hat. Gehe zurück zur App und versuche es erneut.
oauth_error_invalid_client = Diese App ist hier nicht registriert.
oauth_error_invalid_scope = Die App fragt nach Zugriff, den tuita nicht anbietet, oder nach mehr, als sie registriert hat.
oauth_error_unsupported_response_type = Die App fragt nach einer Anmeldeart, die tuita nicht unterstützt.
oauth_error_rate = Du hast zu viele Anfragen beantwortet. Warte ein paar Minuten und versuche es erneut.
oauth_error_unreachable = tuita ist nicht erreichbar. Prüfe deine Verbindung und versuche es erneut.
oauth_home = Zu tuita
share_send_chat = Per Chat senden
share_download = Medien herunterladen
share_download_n = Medien { $number } herunterladen
share_chat_title = Beitrag per Chat senden
share_chat_sent = An { $handle } gesendet
hover_card_label = Profilkarte von { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Medientyp
profile_media_all = Alle Medien
profile_media_photos = Fotos
profile_media_videos = Videos
profile_empty_videos_title = Noch keine Videos
profile_empty_videos_body = Beiträge mit Videos erscheinen hier.
account_add_existing = Bestehendes Konto hinzufügen
rail_label = Suche und Vorschläge
rail_search_submit = Suchen
rail_show_more = Mehr anzeigen
rail_footer = Weitere Links
chat_new = Neuer Chat
chat_search = Unterhaltungen durchsuchen
chat_search_none = Keine Unterhaltung passt zu deiner Suche.
chat_start_title = Unterhaltung beginnen
chat_start_body = Wähle eine Unterhaltung aus der Liste oder beginne eine neue.
explore_sections = Bereiche von Entdecken
explore_see_more = Mehr anzeigen
history_title = Verlauf
history_tabs = Verlaufsbereiche
history_likes_label = Gelikte Beiträge
video_play = Wiedergabe
video_pause = Pause
video_mute = Stummschalten
video_unmute = Ton einschalten
video_volume = Lautstärke
video_seek = Spulen
video_position = { $current } von { $total }
video_speed = Wiedergabegeschwindigkeit
video_fullscreen = Vollbild
video_exit_fullscreen = Vollbild beenden
video_pip = Bild-in-Bild
video_exit_pip = Bild-in-Bild beenden
video_error = Dieses Video kann nicht abgespielt werden.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Seitenleiste ausblenden
lightbox_show_panel = Seitenleiste einblenden
lightbox_view_post = Beitrag ansehen
lightbox_panel = Beitrag und Antworten
emotepick_tabs = Emoji-Sets
emotepick_tab_emoji = Emojis
emotepick_tab_custom = Eigene
admin_emote_tab = Eigene Emojis
admin_emote_title = Eigene Emojis
admin_emote_desc = Emotes gehören zur ganzen Instanz. Alle können sie als :shortcode: in Beiträgen, Namen und Nachrichten verwenden. Ein Kürzel lässt sich nicht umbenennen; lösche das Emote und lade es neu hoch.
admin_emote_shortcode = Kürzel
admin_emote_shortcode_hint = 2 bis 32 Buchstaben, Ziffern oder Unterstriche. Wird als :shortcode: angezeigt.
admin_emote_invalid = Nur Buchstaben, Ziffern und Unterstriche verwenden, 2 bis 32 Zeichen.
admin_emote_category = Kategorie
admin_emote_category_hint = Optional. Gruppiert Emotes in der Auswahl.
admin_emote_file = Emote-Bild
admin_emote_file_hint = PNG, JPEG, WebP oder GIF, bis 256 KB. Datei hierher ziehen oder auswählen.
admin_emote_file_type = Diese Datei ist kein PNG-, JPEG-, WebP- oder GIF-Bild.
admin_emote_file_size = Das Bild ist zu groß (max. 256 KB).
admin_emote_choose_file = Bild auswählen
admin_emote_change_file = Bild ändern
admin_emote_upload = Emote hochladen
admin_emote_upload_failed = Das Emote konnte nicht hochgeladen werden. Versuch es erneut.
admin_emote_uploaded = :{ $shortcode }: hinzugefügt.
admin_emote_list = Emotes dieser Instanz
admin_emote_none = Noch keine eigenen Emotes.
admin_emote_no_category = Keine Kategorie
admin_emote_category_for = Kategorie von :{ $shortcode }:
admin_emote_saved = Kategorie gespeichert.
admin_emote_delete = :{ $shortcode }: löschen
admin_emote_delete_title = Dieses Emote löschen?
admin_emote_delete_body = :{ $shortcode }: wird überall nicht mehr als Bild dargestellt, sondern als einfacher Text. Beiträge, die es verwenden, behalten den Text.
admin_emote_delete_confirm = Emote löschen
admin_emote_deleted = Emote gelöscht.
admin_act_log_emoji_create = Emoji hinzugefügt
admin_act_log_emoji_update = Emoji geändert
admin_act_log_emoji_delete = Emoji gelöscht
e2ee_settings = Verschlüsselte Nachrichten
e2ee_settings_desc = Passphrase, dieses Gerät und Zurücksetzen
e2ee_status_label = Zustand
e2ee_status_none = Aus. Schalte es ein, um Ende-zu-Ende-verschlüsselte Nachrichten zu senden und zu lesen.
e2ee_status_locked = Auf diesem Gerät gesperrt. Gib deine Passphrase ein, um deine Nachrichten hier zu lesen.
e2ee_status_ready = Eingeschaltet und auf diesem Gerät entsperrt.
e2ee_keys_error = Deine Verschlüsselungsschlüssel konnten nicht geprüft werden.
e2ee_setup_title = Verschlüsselte Nachrichten einschalten
e2ee_setup_body = Nachrichten an Leute auf diesem Server werden in deinem Browser verschlüsselt. Der Server speichert nur Text, den er nicht lesen kann. Mit einer Passphrase entsperrst du deine Nachrichten auf einem anderen Gerät.
e2ee_setup_warning = Deine Passphrase lässt sich nicht wiederherstellen. Wenn du sie vergisst, kannst du die Verschlüsselung zurücksetzen, aber deine früheren Nachrichten gehen verloren.
e2ee_setup_submit = Einschalten
e2ee_setup_done = Verschlüsselte Nachrichten sind eingeschaltet
e2ee_unlock_title = Verschlüsselte Nachrichten entsperren
e2ee_unlock_body = Gib deine Passphrase ein, um verschlüsselte Nachrichten auf diesem Gerät zu lesen und zu senden.
e2ee_unlock_submit = Entsperren
e2ee_unlock_wrong = Diese Passphrase ist falsch.
e2ee_unlock_done = Entsperrt
e2ee_passphrase = Deine Passphrase
e2ee_passphrase_confirm = Passphrase bestätigen
e2ee_passphrase_hint = Mindestens 10 Zeichen.
e2ee_passphrase_short = Verwende mindestens 10 Zeichen.
e2ee_passphrase_mismatch = Die Passphrases stimmen nicht überein.
e2ee_failed = Etwas ist schiefgelaufen. Versuche es noch einmal.
e2ee_unsupported = Dieser Browser unterstützt keine verschlüsselten Nachrichten.
e2ee_forget = Auf diesem Gerät vergessen
e2ee_forget_hint = Entfernt deinen Schlüssel und die Kontakte, denen du vertraust, aus diesem Browser. Mit deiner Passphrase kannst du sie wieder entsperren.
e2ee_forget_done = Auf diesem Gerät vergessen
e2ee_reset = Verschlüsselung zurücksetzen
e2ee_reset_hint = Erzeugt einen neuen Schlüssel. Nachrichten, die du vorher gesendet und empfangen hast, werden unlesbar.
e2ee_reset_title = Verschlüsselung zurücksetzen?
e2ee_reset_body = Deine früheren Nachrichten werden unlesbar, für dich und für die Leute, denen du geschrieben hast, und lassen sich nicht zurückholen. Deine Kontakte werden gebeten, deinem neuen Schlüssel zu vertrauen. Wähle eine neue Passphrase.
e2ee_reset_submit = Zurücksetzen
e2ee_reset_done = Die Verschlüsselung wurde zurückgesetzt
e2ee_note_sealed = Ende-zu-Ende-verschlüsselt
e2ee_note_remote = Nicht Ende-zu-Ende-verschlüsselt: Diese Person ist auf einem anderen Server.
e2ee_note_plain = Nicht Ende-zu-Ende-verschlüsselt.
e2ee_plain_mark = Unverschlüsselt
e2ee_preview = Verschlüsselte Nachricht
e2ee_line_sealed = Verschlüsselte Nachricht
e2ee_line_own_key = An einen Schlüssel gesendet, den du nicht mehr hast
e2ee_line_peer_key = Mit einem Schlüssel gesendet, dem du noch nicht vertraut hast
e2ee_line_unreadable = Konnte nicht geöffnet werden
e2ee_line_unsupported = Dieser Browser kann verschlüsselte Nachrichten nicht öffnen
e2ee_block_peer_off = { $handle } hat verschlüsselte Nachrichten noch nicht eingeschaltet.
e2ee_block_self_off = Schalte zuerst in den Einstellungen verschlüsselte Nachrichten ein.
e2ee_block_unlock = Entsperre zuerst in den Einstellungen die verschlüsselten Nachrichten.
e2ee_block_downgrade = { $handle } hat schon früher verschlüsselte Nachrichten genutzt. Diese Unterhaltung fällt nicht auf unverschlüsselte Nachrichten zurück.
e2ee_block_changed = Vertraue dem neuen Schlüssel von { $handle }, um dieser Person zu schreiben.
e2ee_block_error = Deine Verschlüsselungsschlüssel konnten nicht geprüft werden.
e2ee_changed_title = Der Verschlüsselungsschlüssel von { $name } hat sich geändert
e2ee_changed_body = Die Person hat die Verschlüsselung womöglich zurückgesetzt oder ein neues Gerät eingerichtet, oder jemand schaltet sich dazwischen. Vergleiche die neue Sicherheitsnummer mit ihr, bevor du ihr vertraust. Bis dahin kannst du nichts senden.
e2ee_changed_old = Sicherheitsnummer, der du vertraut hast
e2ee_changed_new = Neue Sicherheitsnummer
e2ee_trust = Neuem Schlüssel vertrauen
e2ee_safety_open = Sicherheitsnummer
e2ee_safety_title = Sicherheitsnummer
e2ee_safety_body = Vergleiche diese Nummer persönlich oder in einem Anruf mit { $name }, nicht in diesem Chat. Stimmt sie auf beiden Bildschirmen überein, steht niemand zwischen euch.

## 0031
composer_upload_failed = Hochladen fehlgeschlagen. Versuche es erneut.
composer_send_failed = tuita ist nicht erreichbar. Prüfe deine Verbindung und versuche es erneut.
composer_open_failed = Der Beitragseditor ließ sich nicht öffnen. Versuche es erneut.
