app_name = tuita
loading = Laden
close = Sluiten
dismiss = Sluiten
notifications = Meldingen
avatar_more = nog { $count }
char_counter_left =
    { $count ->
        [one] Nog { $count } teken
       *[other] Nog { $count } tekens
    }
char_counter_over =
    { $count ->
        [one] { $count } teken over de limiet
       *[other] { $count } tekens over de limiet
    }
not_found_title = Hier is niets
not_found_body = Deze pagina bestaat niet of is verplaatst.
not_found_back = Terug
tagline = Elk bericht, twee inkten.
error_title = Er ging iets mis
cancel = Annuleren
retry = Opnieuw proberen
back = Terug
view = Bekijken
follow = Volgen
following = Volgend
follows_you = Volgt jou
verified = Geverifieerd account
bot_account = Geautomatiseerd account
bot_label = Bot
link_copied = Link naar klembord gekopieerd
text_copied = Tekst gekopieerd
time_now = nu
time_minutes = { $n } min
time_hours = { $n } u
time_days = { $n } d
nav_main = Hoofdmenu
nav_home = Home
nav_explore = Verkennen
nav_notifications = Meldingen
nav_bookmarks = Bladwijzers
nav_profile = Profiel
nav_settings = Instellingen
nav_compose = Plaatsen
nav_menu = Menu
badge_unread = { $count } ongelezen
account_menu = Accounts
account_switch = Van account wisselen
account_switch_to = Wisselen naar { $handle }
account_add = Account toevoegen
account_logout = { $handle } uitloggen
account_logout_all = Alle accounts uitloggen
home_feed = Tijdlijn
home_for_you = Voor jou
home_following = Volgend
home_empty_title = Hier is nog niets
home_empty_body = Plaats als eerste iets.
home_following_empty_title = Het is stil in je kring
home_following_empty_body = Volg mensen om deze tijdlijn te vullen met berichten en reposts.
home_find_people = Mensen zoeken
new_posts =
    { $count ->
        [one] { $count } nieuw bericht tonen
       *[other] { $count } nieuwe berichten tonen
    }
new_posts_many = 99+ nieuwe berichten tonen
composer_title = Nieuw bericht
composer_reply_title = Beantwoorden
composer_quote_title = Bericht citeren
composer_placeholder = Wat gebeurt er?
composer_reply_placeholder = Plaats je antwoord
composer_text_label = Berichttekst
composer_replying_to = Antwoord aan { $handle }
composer_cw_label = Inhoudswaarschuwing
composer_cw_placeholder = Inhoudswaarschuwing, bijv. spoilers
composer_cw_toggle = Inhoudswaarschuwing toevoegen
composer_sensitive_toggle = Media als gevoelig markeren
composer_options = Berichtopties
composer_policy_label = Wie kan antwoorden
composer_suggestions = Suggesties
composer_tag_posts =
    { $count ->
        [one] { $compact } bericht
       *[other] { $compact } berichten
    }
composer_post = Plaatsen
composer_reply = Beantwoorden
policy_everyone = Iedereen kan antwoorden
policy_following = Mensen die je volgt kunnen antwoorden
policy_mentioned = Alleen mensen die je noemt kunnen antwoorden
post_sent = Bericht verzonden
post_by = Bericht van { $name }
post_reposted_by = { $name } repostte
post_pinned = Vastgemaakt bericht
post_replying_to = Antwoord aan { $handle }
post_replying_unavailable = Antwoord op een bericht dat niet beschikbaar is
post_unavailable = Dit bericht is niet beschikbaar.
post_menu = Meer
post_copy_link = Link kopiëren
post_copy_text = Tekst kopiëren
post_pin = Vastzetten op profiel
post_unpin = Losmaken van profiel
post_pinned_toast = Vastgezet op je profiel
post_unpinned = Losgemaakt van je profiel
post_delete = Bericht verwijderen
post_delete_title = Bericht verwijderen?
post_delete_body = Dit kan niet ongedaan worden gemaakt. Het wordt verwijderd van je profiel, uit de tijdlijnen van je volgers en uit de zoekresultaten.
post_deleted = Bericht verwijderd
post_view_profile = { $handle } bekijken
post_mute = { $handle } dempen
post_block = { $handle } blokkeren
post_reply_label = Antwoorden ({ $count })
post_reply_following_only = Alleen mensen die de auteur volgt kunnen antwoorden.
post_reply_mentioned_only = Alleen mensen die in dit bericht worden genoemd kunnen antwoorden.
post_repost_menu = Repostopties
post_repost = Reposten
post_undo_repost = Repost ongedaan maken
post_quote = Citeren
post_reposts_label = Reposts en citaten ({ $count })
post_like_label = Leuk vinden ({ $count })
post_bookmark_label = Bladwijzer
post_share = Delen of link kopiëren
cw_show = Tonen
cw_hide = Verbergen
cw_label = Inhoudswaarschuwing: { $reason }
muted_toast = { $handle } gedempt
unmuted_toast = Demping van { $handle } opgeheven
blocked_toast = { $handle } geblokkeerd
unblocked_toast = Blokkade van { $handle } opgeheven
block_title = { $handle } blokkeren?
block_body = Deze persoon kan je niet volgen en je ziet hun berichten niet. Ze krijgen hier geen melding van.
block_confirm = Blokkeren
notify_like = @{ $handle } vindt je bericht leuk
notify_repost = @{ $handle } repostte je bericht
notify_reply = @{ $handle } antwoordde je
notify_mention = @{ $handle } noemde je
notify_quote = @{ $handle } citeerde je bericht
notify_follow = @{ $handle } volgt je nu
notify_follow_request = @{ $handle } wil je volgen
notify_poll = Een peiling waaraan je meedeed is afgelopen
notify_affiliation = @{ $handle } nodigt je uit om je aan te sluiten
shortcuts_title = Sneltoetsen
shortcut_group_navigation = Navigatie
shortcut_group_actions = Acties
shortcut_or = /
shortcut_then = dan
shortcut_plus = +
shortcut_next_prev = Volgend / vorig bericht
shortcut_open = Bericht openen
shortcut_search = Zoeken
shortcut_go_home = Home
shortcut_go_explore = Verkennen
shortcut_go_notifications = Meldingen
shortcut_go_bookmarks = Bladwijzers
shortcut_go_profile = Je profiel
shortcut_go_settings = Instellingen
shortcut_new_post = Nieuw bericht
shortcut_like = Leuk vinden
shortcut_reply = Beantwoorden
shortcut_repost = Reposten of citeren
shortcut_bookmark = Bladwijzer
shortcut_send = Bericht verzenden
shortcut_help = Deze hulp tonen
auth_login_title = Inloggen
auth_add_title = Nog een account toevoegen
auth_add_sub = Log in om op dit apparaat tussen maximaal { $max } accounts te wisselen zonder uit te loggen.
auth_signup_title = Account aanmaken
auth_handle = Gebruikersnaam
auth_password = Wachtwoord
auth_display_name = Weergavenaam
auth_handle_hint = 3–15 letters, cijfers of underscores
auth_password_hint = Minstens 8 tekens
auth_consent_pre = Ik ben minstens { $minAge } en ga akkoord met de{" "}
auth_consent_terms = Gebruiksvoorwaarden
auth_consent_and = {" "}en het{" "}
auth_consent_privacy = Privacybeleid
auth_consent_end = {"."}
auth_consent_required = Accepteer de gebruiksvoorwaarden en het privacybeleid.
auth_signin = Inloggen
auth_signin_add = Inloggen en wisselen
auth_create = Account aanmaken
auth_to_signup = Nieuw hier? Maak een account aan
auth_to_login = Al een account? Log in
auth_closed = Registreren is op deze instance momenteel gesloten.
landing_nav = Voettekst
landing_rules = Huisregels
landing_stats = Op dit moment
landing_people = Mensen
landing_posts = Berichten
landing_clock = Jouw lokale tijd
landing_trends = Trending
landing_trend =
    { $count ->
        [one] { $count } bericht
       *[other] { $count } berichten
    }
landing_join = Doe mee met { $name }
landing_join_sub = Maak een account en plaats berichten in twee inkten.
landing_closed = Registreren is momenteel gesloten. Heb je al een account? Log in.
landing_terms = Voorwaarden
landing_privacy = Privacy
landing_source = Broncode
landing_powered = Mogelijk gemaakt door tuita { $version }
docs_terms_title = Gebruiksvoorwaarden
docs_privacy_title = Privacybeleid
docs_updated = Laatst bijgewerkt op { $date }
docs_date = 5 november 2026
docs_see_privacy = Zie ook het privacybeleid.
docs_see_terms = Zie ook de gebruiksvoorwaarden.
docs_contact_fallback = de beheerders van deze instance (via een direct bericht of een melding)
docs_terms_agreement_h = 1. Overeenkomst
docs_terms_agreement_p = Deze gebruiksvoorwaarden (“Voorwaarden”) gelden voor je gebruik van { $name }, de tuita-instance op { $domain } (“de dienst”, “wij”). Door een account aan te maken of de dienst te gebruiken ga je akkoord met deze Voorwaarden en met ons privacybeleid. Ga je er niet mee akkoord, gebruik de dienst dan niet.
docs_terms_eligibility_h = 2. Wie de dienst mag gebruiken
docs_terms_eligibility_p = Je moet minstens { $minAge } jaar oud zijn, of de minimumleeftijd voor digitale toestemming in je woonland als die hoger is. Je moet een bindende overeenkomst kunnen aangaan en volgens de toepasselijke wetgeving niet van het gebruik van de dienst zijn uitgesloten.
docs_terms_account_h = 3. Je account
docs_terms_account_1 = Houd je wachtwoord geheim. Je bent verantwoordelijk voor de activiteit op je account.
docs_terms_account_2 = Je gebruikersnaam mag zich niet voordoen als een andere persoon of organisatie en mag mensen niet misleiden over wie je bent.
docs_terms_account_3 = Laat het ons snel weten als je denkt dat je account gehackt is. Je kunt andere sessies uitloggen via Instellingen → Account.
docs_terms_content_h = 4. Je inhoud
docs_terms_content_p1 = Wat je plaatst blijft van jou. Je geeft ons een wereldwijde, niet-exclusieve, kosteloze licentie om je inhoud op te slaan, weer te geven, te reproduceren en te verspreiden, uitsluitend om de dienst te laten werken en te verbeteren. Dat omvat het verzenden naar gefedereerde tuita-instances (zie sectie 6). Deze licentie eindigt wanneer je de inhoud of je account verwijdert, behalve voor kopieën die al bij andere instances zijn bezorgd of door anderen zijn bewaard.
docs_terms_content_p2 = Je bent verantwoordelijk voor wat je plaatst. Plaats alleen inhoud die je mag delen.
docs_terms_rules_h = 5. Community-regels
docs_terms_rules_lead = Door { $name } te gebruiken ga je ermee akkoord de wet en deze regels niet te overtreden:
docs_terms_rules_p = Je gaat er ook mee akkoord de dienst niet aan te vallen, bijvoorbeeld door beveiligingsmaatregelen te omzeilen, de dienst op grote schaal uit te lezen, malware te versturen of de dienst te overbelasten.
docs_terms_federation_h = 6. Federatie
docs_terms_federation_p = { $name } kan verbinding maken (“federeren”) met andere onafhankelijke tuita-instances die de beheerders goedkeuren. Als je met mensen op een andere instance communiceert, worden je openbare profiel, openbare berichten en interacties (bijvoorbeeld volgen, liken, reposten, antwoorden of een direct bericht) naar die instance verzonden. Andere instances worden door anderen beheerd en hebben hun eigen voorwaarden. Wij hebben geen controle over inhoud die een andere instance heeft ontvangen en kunnen die niet verwijderen, al sturen we wel verwijderverzoeken als je iets verwijdert.
docs_terms_moderation_h = 7. Moderatie en handhaving
docs_terms_moderation_p = Moderators kunnen inhoud verwijderen, functies beperken of accounts opschorten die deze Voorwaarden of de wet overtreden. Waar mogelijk leggen we uit waarom. Iedereen kan een bericht of account melden. Meldingen gaan naar de moderators van deze instance. We kunnen op elk moment stoppen met federeren met een instance.
docs_terms_termination_h = 8. Vertrekken
docs_terms_termination_p = Je kunt je gegevens op elk moment exporteren of je account verwijderen via Instellingen → Account. We kunnen je toegang opschorten of beëindigen als je deze Voorwaarden ernstig of herhaaldelijk overtreedt. We kunnen de dienst ook sluiten en proberen je dat dan tijdig van tevoren te laten weten.
docs_terms_disclaimers_h = 9. Uitsluitingen en aansprakelijkheid
docs_terms_disclaimers_p = De dienst wordt geleverd “zoals hij is” en “zoals beschikbaar”, zonder enige garantie, voor zover de wet dat toestaat. Wij zijn niet aansprakelijk voor indirecte, incidentele of gevolgschade, of voor inhoud die door gebruikers of andere instances is geplaatst. Niets in deze Voorwaarden beperkt aansprakelijkheid die wettelijk niet beperkt kan worden, of wettelijke consumentenrechten.
docs_terms_changes_h = 10. Wijzigingen
docs_terms_changes_p = We kunnen deze Voorwaarden bijwerken. Bij belangrijke wijzigingen laten we dit in de app weten voordat ze ingaan. Als je de dienst daarna blijft gebruiken, accepteer je de nieuwe Voorwaarden.
docs_terms_contact_h = 11. Contact
docs_terms_contact_lead = Vragen over deze Voorwaarden:
docs_privacy_summary_h = De korte versie
docs_privacy_summary_1 = Geen advertenties, geen trackingscripts, geen verkoop van gegevens.
docs_privacy_summary_2 = Eén cookie, om je ingelogd te houden. Thema-instellingen blijven in je browser.
docs_privacy_summary_3 = Openbare berichten zijn openbaar en kunnen gefedereerde tuita-instances bereiken.
docs_privacy_summary_4 = Directe berichten tussen accounts op deze server zijn end-to-end versleuteld zodra beide mensen dat aanzetten: ze worden opgeslagen als tekst die de server niet kan lezen, dus een databaselek of een blik op de opgeslagen gegevens onthult niets. Berichten aan mensen op andere servers en berichten die zijn verstuurd voordat je versleuteling aanzette, zijn op deze manier niet versleuteld en beheerders kunnen ze technisch gezien lezen. Deze server levert ook de app die versleutelt, dus een beheerder die die app aanpast, zou kunnen meekijken met wat je typt.
docs_privacy_summary_5 = Je kunt je gegevens op elk moment exporteren of verwijderen.
docs_privacy_controller_h = 1. Wie wij zijn
docs_privacy_controller_p = Dit beleid geldt voor { $name } op { $domain }, beheerd door de beheerders van de instance. Zij bepalen hoe je gegevens worden verwerkt.
docs_privacy_collect_h = 2. Wat we verzamelen
docs_privacy_collect_1 = Accountgegevens: gebruikersnaam, weergavenaam, wachtwoord (alleen opgeslagen als gezouten argon2id-hash) en je profielgegevens: bio, locatie, website, avatar, banner, vorm en kleur.
docs_privacy_collect_2 = Inhoud: berichten, afbeeldingen, antwoorden, likes, reposts, bladwijzers, volgrelaties, blokkades, dempingen en meldingen.
docs_privacy_collect_3 = Directe berichten: opgeslagen op deze server zodat ze aan jou en de andere persoon kunnen worden bezorgd. Een end-to-end versleuteld bericht wordt opgeslagen als versleutelde tekst die we niet kunnen lezen; wie het stuurde, naar wie en wanneer blijft voor ons zichtbaar. Andere berichten worden opgeslagen als platte tekst. Je versleutelingssleutel wordt hier alleen bewaard in een vorm die door je wachtwoordzin is vergrendeld en die wij nooit zien.
docs_privacy_collect_4 = Sessiegegevens: per ingelogd apparaat de user-agent van de browser, het IP-adres en het tijdstip van laatste gebruik, zodat je sessies kunt bekijken en uitloggen.
docs_privacy_collect_5 = Kortlevende beveiligingsgegevens: IP-adressen die enkele minuten in het geheugen blijven voor snelheidsbeperking.
docs_privacy_collect_p = We verzamelen geen e-mailadres, telefoonnummer, contacten of locatie, tenzij je die zelf in je profiel invult.
docs_privacy_use_h = 3. Hoe we het gebruiken
docs_privacy_use_p = Alleen om de dienst te laten werken: je profiel en berichten tonen, tijdlijnen en meldingen opbouwen, directe berichten bezorgen, accounts beveiligen, misbruik voorkomen en de Voorwaarden handhaven. De tijdlijn “Voor jou” rangschikt berichten op likes, reposts, antwoorden en leeftijd. Er wordt geen profiel van je opgebouwd.
docs_privacy_visibility_h = 4. Wat anderen kunnen zien
docs_privacy_visibility_p = Je profiel, berichten, antwoorden, reposts, likes, volgers en wie je volgt zijn openbaar voor iedereen die is ingelogd op deze instance en op gefedereerde instances. Wanneer iemand een link naar een bericht of profiel deelt, kunnen apps als Discord, Slack of iMessage een voorbeeld ophalen met het openbare bericht, de afbeeldingen en de openbare profielgegevens. Berichten met een inhoudswaarschuwing of die als gevoelig zijn gemarkeerd, worden zonder tekst en afbeeldingen getoond. Bladwijzers, blokkades, dempingen, meldingen en instellingen zijn privé. Als je “Toon mij in suggesties” uitzet, kunnen mensen je nog steeds vinden via je exacte gebruikersnaam.
docs_privacy_federation_h = 5. Federatie
docs_privacy_federation_p = Wanneer je iemand op een andere tuita-instance volgt, beantwoordt, noemt, liket, repost of een bericht stuurt, sturen we de benodigde gegevens naar die instance: je openbare profiel, het bericht of de interactie en bij directe berichten de berichttekst. Directe berichten naar andere instances zijn niet end-to-end versleuteld. De instances van je volgers ontvangen je openbare berichten en profielwijzigingen. Die instances worden zelfstandig beheerd en bewaren gegevens volgens hun eigen beleid. Wanneer je een bericht of je account verwijdert, vragen we gefedereerde instances hun kopieën te verwijderen, maar we kunnen niet garanderen dat ze dat doen.
docs_privacy_cookies_h = 6. Cookies en lokale opslag
docs_privacy_cookies_p = We gebruiken één strikt noodzakelijke HttpOnly-sessiecookie. Je thema-, kleur-, tekstgrootte- en bewegingsvoorkeuren worden opgeslagen in de lokale opslag van je browser en nooit naar ons verzonden.
docs_privacy_retention_h = 7. Hoe lang we gegevens bewaren
docs_privacy_retention_1 = Inhoud en accountgegevens worden bewaard totdat je ze verwijdert.
docs_privacy_retention_2 = Verwijderde berichten verdwijnen direct uit het openbare zicht en tekst en afbeeldingen worden gewist.
docs_privacy_retention_3 = Wanneer je je account verwijdert, wissen we je berichten, likes, volgrelaties, directe berichten en sessies en anonimiseren we het accountrecord.
docs_privacy_retention_4 = Sessies verlopen na 30 dagen, of meteen zodra je uitlogt.
docs_privacy_retention_5 = Moderatiemeldingen en het auditlogboek van beheerders worden bewaard zolang dat nodig is om de dienst veilig te houden.
docs_privacy_rights_h = 8. Jouw rechten
docs_privacy_rights_p = Afhankelijk van waar je woont (bijvoorbeeld op grond van de AVG of CCPA) heb je mogelijk het recht om je gegevens in te zien, te corrigeren, te exporteren of te verwijderen en om bezwaar te maken tegen de verwerking of die te laten beperken. Het meeste kun je zelf doen via Instellingen → Account: je profiel bewerken, je gegevens downloaden en je account verwijderen. Neem voor al het andere contact op met:
docs_privacy_rights_end = Je kunt ook een klacht indienen bij je lokale gegevensbeschermingsautoriteit.
docs_privacy_security_h = 9. Beveiliging
docs_privacy_security_p = Wachtwoorden worden gehasht met scrypt. Sessietokens worden alleen als hash opgeslagen. Verzoeken tussen gefedereerde instances worden ondertekend met Ed25519-sleutels. Geen enkel systeem is volledig veilig, dus gebruik een uniek wachtwoord.
docs_privacy_children_h = 10. Kinderen
docs_privacy_children_p = De dienst is niet bedoeld voor kinderen onder de minimumleeftijd uit de Voorwaarden. Denk je dat een kind een account heeft, meld het dan.
docs_privacy_changes_h = 11. Wijzigingen
docs_privacy_changes_p = We plaatsen wijzigingen in dit beleid hier en werken de datum hierboven bij. Belangrijke wijzigingen kondigen we aan in de app.
docs_privacy_contact_h = 12. Contact
docs_privacy_contact_lead = Privacyvragen of -verzoeken:
profile_posts =
    { $count ->
        [one] { $compact } bericht
       *[other] { $compact } berichten
    }
profile_edit = Profiel bewerken
profile_more = Meer
profile_copy_link = Profiellink kopiëren
profile_mute = { $handle } dempen
profile_unmute = Demping van { $handle } opheffen
profile_block = { $handle } blokkeren
profile_unblock = Blokkade van { $handle } opheffen
profile_unblock_button = Deblokkeren
profile_suspended = Dit account is opgeschort. Alleen medewerkers kunnen het zien.
profile_muted = Gedempt
profile_org = Organisatie
profile_role_admin = Beheerder
profile_role_moderator = Mod
profile_joined = Lid sinds { $date }
profile_following_label = Volgend
profile_followers =
    { $count ->
        [one] Volger
       *[other] Volgers
    }
profile_counts = Aantallen volgers
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Gevolgd door { $a }
            }
        [2]
            { $others ->
               *[other] Gevolgd door { $a } en { $b }
            }
       *[other]
            { $others ->
                [one] Gevolgd door { $a }, { $b } en { $others } ander die je volgt
               *[other] Gevolgd door { $a }, { $b } en { $others } anderen die je volgt
            }
    }
profile_banner = Profielbanner
profile_tabs = Profielonderdelen
profile_tab_posts = Berichten
profile_tab_replies = Antwoorden
profile_tab_media = Media
profile_tab_likes = Likes
profile_posts_label = Berichten
profile_empty_posts_self_title = Je hebt nog niets geplaatst
profile_empty_posts_self_body = Je berichten en reposts verschijnen hier.
profile_empty_posts_title = { $handle } heeft nog niets geplaatst
profile_empty_posts_body = Zodra dat gebeurt, verschijnen de berichten hier.
profile_empty_replies_title = Nog geen antwoorden
profile_empty_replies_body = Antwoorden verschijnen hier.
profile_empty_media_title = Nog geen foto’s
profile_empty_media_body = Berichten met afbeeldingen verschijnen hier.
profile_empty_likes_title = Nog geen likes
profile_empty_likes_body = Berichten die je leuk vindt verschijnen hier.
profile_blocked_title = Je hebt { $handle } geblokkeerd
profile_blocked_body = Je ziet hun berichten niet en zij kunnen je niet volgen of berichten sturen.
profile_hidden_title = Je kunt deze berichten niet zien
profile_hidden_body = { $handle } heeft beperkt wie het profiel kan zien.
follows_tabs = Volglijsten
follows_tab_followers = Volgers
follows_tab_following = Volgend
follows_label_followers = Volgers
follows_label_following = Volgend
follows_empty_followers_title = Nog geen volgers
follows_empty_followers_body = Wanneer mensen dit account volgen, verschijnen ze hier.
follows_empty_following_title = Volgt niemand
follows_empty_following_body = Accounts die deze persoon volgt verschijnen hier.
thread_title = Bericht
thread_doc_title = { $name }: “{ $text }”
thread_views =
    { $count ->
        [one] Weergave
       *[other] Weergaven
    }
thread_reposts =
    { $count ->
        [one] Repost
       *[other] Reposts
    }
thread_quotes =
    { $count ->
        [one] Citaat
       *[other] Citaten
    }
thread_likes =
    { $count ->
        [one] Like
       *[other] Likes
    }
thread_stats = Berichtstatistieken
thread_replies = Antwoorden
thread_hidden = Verborgen antwoorden ({ $count })
thread_hidden_note = De auteur heeft deze antwoorden verborgen.
thread_list_likes = Leuk gevonden door
thread_list_reposts = Gerepost door
thread_list_quotes = Citaten
thread_list_empty = Nog niemand
tag_empty_title = Geen berichten met { $tag }
tag_empty_body = Wees de eerste die het gebruikt.
tag_posts_label = Berichten met { $tag }
bookmarks_title = Bladwijzers
bookmarks_label = Berichten met bladwijzer
bookmarks_empty_title = Bewaar berichten voor later
bookmarks_empty_body = Tik op het bladwijzericoon bij een bericht om het hier te bewaren. Alleen jij ziet je bladwijzers.
settings_account = Je account
settings_account_desc = Profiel en wachtwoord
settings_profile = Profiel bewerken
settings_profile_desc = Naam, bio, avatarvorm en inkt
settings_privacy = Privacy en veiligheid
settings_privacy_desc = Berichten, vindbaarheid, gedempte woorden, geblokkeerde en gedempte accounts
settings_notifications = Meldingen
settings_notifications_desc = Kies waarover je meldingen krijgt
settings_appearance = Weergave
settings_appearance_desc = Thema, inkt, tekstgrootte, contrast en beweging
settings_language = Taal
settings_language_desc = Taal van de interface
settings_language_note = Opgeslagen in deze browser en, als je bent ingelogd, in je account.
settings_about = Over deze instance
settings_about_desc = Regels, voorwaarden, privacy, versie en sneltoetsen
settings_logout = Uitloggen
settings_account_profile = Profiel
settings_account_joined = Lid sinds { $date }
settings_account_edit = Profiel bewerken
settings_password_title = Wachtwoord wijzigen
settings_password_current = Huidig wachtwoord
settings_password_new = Nieuw wachtwoord
settings_password_hint = Minstens 8 tekens. Andere apparaten worden uitgelogd.
settings_password_confirm = Bevestig nieuw wachtwoord
settings_password_submit = Wachtwoord bijwerken
settings_password_mismatch = De nieuwe wachtwoorden komen niet overeen.
settings_password_updated = Wachtwoord bijgewerkt. Andere sessies zijn uitgelogd.
settings_sessions_title = Apparaten en sessies
settings_sessions_desc = Log alles uit wat je niet herkent en wijzig daarna je wachtwoord.
settings_device_on = { $browser } op { $os }
settings_device_unknown = Onbekend apparaat
settings_session_current = Dit apparaat
settings_session_last_active = Laatst actief: { $time }
settings_session_sign_out = Uitloggen
settings_session_sign_out_device = { $device } uitloggen
settings_session_signed_out = Sessie uitgelogd
settings_sessions_revoke_all = Alle andere apparaten uitloggen
settings_sessions_revoke_title = Andere apparaten uitloggen?
settings_sessions_revoke_body = Alle apparaten behalve dit apparaat worden uitgelogd.
settings_sessions_revoke_confirm = Andere uitloggen
settings_sessions_revoked =
    { $count ->
        [one] { $count } sessie uitgelogd
       *[other] { $count } sessies uitgelogd
    }
settings_profile_preview = Voorbeeld
settings_profile_preview_alt = Voorbeeld van avatar
settings_profile_uploads = Uploads van avatar en banner komen later. Tot die tijd gebruiken we je gegenereerde avatar en banner.
settings_profile_details = Gegevens
settings_profile_name = Weergavenaam
settings_profile_bio = Bio
settings_profile_location = Locatie
settings_profile_website = Website
settings_profile_shape = Avatarvorm
settings_profile_ink = Profielinkt
settings_profile_save = Profiel opslaan
settings_profile_saved = Profiel bijgewerkt
settings_shape_square = Vierkant
settings_shape_circle = Cirkel
settings_shape_squircle = Afgerond vierkant
settings_shape_octagon = Achthoek
settings_ink_bone = Beige
settings_ink_phosphor = Fosfor
settings_ink_amber = Amber
settings_ink_cyan = Cyaan
settings_ink_rose = Roze
settings_dm_title = Directe berichten
settings_dm_desc = Wie een gesprek met je kan beginnen
settings_dm_label = Wie je berichten kan sturen
settings_dm_everyone = Iedereen
settings_dm_following = Mensen die ik volg
settings_dm_none = Niemand
settings_dm_saved = Berichtinstellingen opgeslagen
settings_discover_title = Vindbaarheid
settings_discoverable = Toon mij in suggesties en zoekresultaten
settings_discoverable_hint = Mensen kunnen je nog steeds vinden via je exacte gebruikersnaam
settings_show_client = Toon vanaf welk apparaat ik berichten plaats
settings_show_client_hint = Voegt het apparaat toe onder je nieuwe berichten
settings_words_title = Gedempte woorden
settings_words_desc = Verberg berichten met deze woorden, zinnen of #hashtags in je tijdlijnen, zoekresultaten en antwoorden. Je eigen berichten worden nooit verborgen.
settings_words_label = Woord om te dempen
settings_words_placeholder = Voeg een woord, zin of #hashtag toe
settings_words_add = Dempen
settings_words_remove = Demping van { $word } opheffen
settings_words_none = Geen gedempte woorden.
settings_words_limit = Je kunt maximaal { $count } woorden dempen.
settings_blocked_title = Geblokkeerde accounts
settings_blocked_desc = Ze kunnen je niet volgen of berichten sturen en je berichten niet zien of erop reageren. Ze krijgen hier geen melding van.
settings_blocked_empty = Je hebt niemand geblokkeerd.
settings_muted_title = Gedempte accounts
settings_muted_desc = Je ziet hun berichten en meldingen niet. Ze kunnen je nog steeds volgen en berichten sturen.
settings_muted_empty = Je hebt niemand gedempt.
settings_unblock = Deblokkeren
settings_unmute = Dempen opheffen
settings_unblock_user = Blokkade van { $handle } opheffen
settings_unmute_user = Demping van { $handle } opheffen
settings_notif_title = Meldingen ontvangen over
settings_notif_desc = Gedempte en geblokkeerde accounts sturen je nooit meldingen.
settings_notif_like = Likes
settings_notif_like_hint = Als iemand je bericht leuk vindt
settings_notif_repost = Reposts
settings_notif_repost_hint = Als iemand je bericht repost
settings_notif_reply = Antwoorden
settings_notif_reply_hint = Als iemand je antwoordt
settings_notif_mention = Vermeldingen
settings_notif_mention_hint = Als iemand je noemt
settings_notif_quote = Citaten
settings_notif_quote_hint = Als iemand je bericht citeert
settings_notif_follow = Nieuwe volgers
settings_notif_follow_hint = Als iemand je volgt
settings_desktop_title = Bureaubladmeldingen
settings_desktop_desc = Wordt alleen als systeemmelding getoond zolang dit tabblad op de achtergrond staat.
settings_desktop_state = Toestemming: { $state }
settings_desktop_allowed = Toegestaan
settings_desktop_blocked = Geblokkeerd
settings_desktop_not_asked = Niet gevraagd
settings_desktop_enable = Inschakelen
settings_desktop_blocked_toast = Meldingen voor deze site zijn geblokkeerd in je browserinstellingen.
settings_desktop_unsupported = Deze browser ondersteunt geen bureaubladmeldingen.
settings_theme_title = Thema
settings_appearance_note = Opgeslagen op dit apparaat en direct toegepast.
settings_theme_mode = Thema
settings_theme_system = Systeem
settings_theme_light = Licht
settings_theme_dark = Donker
settings_theme_ink = Inkt
settings_theme_contrast = Contrast
settings_contrast_normal = Normaal
settings_contrast_more = Meer
settings_reading_title = Tekst en beweging
settings_theme_text_size = Tekstgrootte
settings_theme_motion = Beweging
settings_motion_system = Systeem
settings_motion_reduce = Verminderen
settings_preview_title = Voorbeeld
settings_preview_text = Zo zien berichten eruit met jouw instellingen.
settings_about_domain = Domein
settings_about_version = Versie
settings_about_contact = Contact
settings_about_terms = Gebruiksvoorwaarden
settings_about_privacy = Privacybeleid
settings_about_source = Broncode
settings_about_rules = Regels
explore_search_label = Zoek berichten, mensen en tags
explore_search_placeholder = Zoek berichten, mensen, #tags
explore_clear = Zoekopdracht wissen
explore_tips = Zoektips
explore_tips_title = Zoekoperatoren
explore_op_from = Berichten van een account
explore_op_to = Antwoorden of vermeldingen aan een account
explore_op_media = Berichten met afbeeldingen (of has:poll, has:links)
explore_op_since = Geplaatst op of na een datum
explore_op_until = Geplaatst op of vóór een datum
explore_op_phrase = Woorden in deze volgorde
explore_op_exclude = Berichten met een woord weglaten
explore_results = Zoekresultaten
explore_tab_posts = Berichten
explore_tab_people = Mensen
explore_tab_media = Media
explore_people = Mensen
explore_posts = Berichten
explore_recent = Recente zoekopdrachten
explore_recent_clear = Wissen
explore_trending = Trending
explore_trend_posts =
    { $count ->
        [one] { $compact } bericht
       *[other] { $compact } berichten
    }
explore_trends_empty = Nog geen trends. Gebruik een #hashtag!
explore_suggestions = Wie je kunt volgen
explore_suggestions_empty = Je volgt iedereen. Indrukwekkend.
explore_no_posts_title = Geen berichten voor “{ $q }”
explore_no_posts_body = Probeer een ander woord of een andere hashtag.
explore_no_people_title = Geen mensen voor “{ $q }”
explore_no_people_body = Probeer een naam, gebruikersnaam of @naam@instance voor iemand op een andere instance.
explore_no_media_title = Geen afbeeldingen voor “{ $q }”
explore_no_media_body = Passende berichten met afbeeldingen verschijnen hier.
notif_filter = Meldingsfilter
notif_tab_all = Alles
notif_tab_mentions = Vermeldingen
notif_empty_all_title = Nog geen meldingen
notif_empty_all_body = Likes, antwoorden, reposts en nieuwe volgers verschijnen hier.
notif_empty_mentions_title = Nog geen vermeldingen
notif_empty_mentions_body = Antwoorden, vermeldingen en citaten van je berichten verschijnen hier.
notif_who_two = { $first } en { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } en { $count } ander
       *[other] { $first }, { $second } en { $count } anderen
    }
notif_verb_like =
    { $count ->
        [one] vindt je bericht leuk
       *[other] vinden je bericht leuk
    }
notif_verb_repost =
    { $count ->
        [one] heeft je bericht gerepost
       *[other] hebben je bericht gerepost
    }
notif_verb_follow =
    { $count ->
        [one] volgt je nu
       *[other] volgen je nu
    }
notif_verb_follow_request =
    { $count ->
        [one] wil je volgen
       *[other] willen je volgen
    }
notif_verb_reply =
    { $count ->
        [one] heeft je geantwoord
       *[other] hebben je geantwoord
    }
notif_verb_mention =
    { $count ->
        [one] heeft je genoemd
       *[other] hebben je genoemd
    }
notif_verb_quote =
    { $count ->
        [one] heeft je bericht geciteerd
       *[other] hebben je bericht geciteerd
    }
notif_verb_poll =
    { $count ->
        [one] heeft een peiling beëindigd waaraan je meedeed
       *[other] hebben een peiling beëindigd waaraan je meedeed
    }
notif_verb_affiliation =
    { $count ->
        [one] nodigt je uit om je aan te sluiten
       *[other] nodigen je uit om je aan te sluiten
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Inplannen
composer_post_all = Alles plaatsen
composer_err_type = Dat bestand is geen afbeelding of video die tuita kan gebruiken (PNG, JPEG, GIF, WebP, MP4, WebM of MOV).
composer_err_image_size = Afbeelding is te groot (max. 10 MB).
composer_err_video_size = Video’s mogen maximaal { $mb } MB zijn.
composer_err_max_images = Je kunt maximaal 4 afbeeldingen toevoegen.
composer_err_exclusive = Een bericht kan een video, afbeeldingen of een peiling bevatten, maar slechts één daarvan.
composer_extra_placeholder = Nog een bericht toevoegen
composer_extra_label = Draadbericht { $n }
composer_extra_remove = Draadbericht { $n } verwijderen
composer_schedule_chip = Wordt geplaatst op { $time }
composer_schedule_clear = Niet inplannen
composer_add_images = Afbeeldingen toevoegen
composer_add_video = Video toevoegen
composer_add_poll = Peiling toevoegen
composer_add_post = Nog een bericht toevoegen
composer_scheduled = Ingepland voor { $time }
composer_thread_sent = Draad van { $count } berichten verzonden
composer_remove_video = Video verwijderen
composer_remove_image = Afbeelding verwijderen
composer_uploading = Uploaden
composer_uploading_video = Video uploaden… { $percent }%
composer_alt_video = Beschrijf deze video
composer_alt_image = Beschrijf deze afbeelding
composer_alt_label = Beschrijving (alt-tekst) van bijlage { $index }
composer_emoji = Emoji toevoegen
poll_length_hours = { $n } u
poll_length_days = { $n } d
poll_choice = Keuze { $n }
poll_choice_optional = Keuze { $n } (optioneel)
poll_remove_choice = Keuze { $n } verwijderen
poll_add_choice = Keuze toevoegen
poll_length = Duur van de peiling
poll_multiple_label = Meerdere keuzes toestaan
poll_remove = Peiling verwijderen
schedule_title = Bericht inplannen
schedule_hour = Over een uur
schedule_evening = Vanavond
schedule_tomorrow = Morgenochtend
schedule_week = Over een week
schedule_window = Kies een tijd tussen een minuut en 90 dagen vanaf nu.
schedule_custom = Kies datum en tijd
schedule_set = Instellen
schedule_clear = Niet inplannen
schedule_view = Ingeplande berichten bekijken
dropzone_drop = Loslaten om toe te voegen
dropzone_reject = Dit bestand kan niet worden gebruikt
nav_messages = Berichten
nav_lists = Lijsten
nav_scheduled = Ingepland
notify_message = @{ $handle } stuurde je een bericht
shortcut_go_messages = Berichten
shortcut_go_lists = Lijsten
cropper_stage = Uitsnedegebied. Sleep om de afbeelding te verplaatsen. Gebruik de pijltjestoetsen om te schuiven en plus en min om te zoomen.
cropper_hint = Sleep om te verplaatsen. Knijp, scroll of gebruik de schuifregelaar om te zoomen.
cropper_zoom = Zoomen
cropper_zoom_in = Inzoomen
cropper_zoom_out = Uitzoomen
cropper_rotate = Draaien
cropper_reset = Herstellen
cropper_apply = Toepassen
image_field_avatar = Profielfoto
image_field_banner = Banner
image_field_edit_avatar = Profielfoto bewerken
image_field_edit_banner = Banner bewerken
image_field_change = Wijzigen
image_field_remove = Verwijderen
image_field_hint = PNG, JPEG, GIF of WebP tot 10 MB. Een GIF wordt een stilstaande afbeelding.
image_field_not_image = Dat bestand is geen afbeelding die tuita kan gebruiken (PNG, JPEG, GIF of WebP).
image_field_too_large = Afbeelding is te groot (max. 10 MB).
image_field_uploading = Uploaden
image_field_saved = Afbeelding opgeslagen.
image_field_removed = Afbeelding verwijderd.
edit_title = Bericht bewerken
edit_note = Je kunt een bericht tot 30 minuten na plaatsing bewerken. Mensen kunnen zien dat een bericht is bewerkt, en ook de eerdere versies.
edit_save = Opslaan
edit_saved = Bericht bijgewerkt
edit_edited = Bewerkt
edit_history_title = Bewerkingsgeschiedenis
edit_history_current = Huidige versie
edit_history_earlier = Eerdere versie
edit_history_original = Origineel
report_post_title = Bericht melden
report_user_title = { $handle } melden
report_intro = Wat is er mis? De moderators van deze instance beoordelen je melding. { $handle } ziet niet wie de melding heeft gedaan.
report_reason = Reden
report_note = Nog iets? (optioneel)
report_send = Melding versturen
report_sent = Bedankt. Je melding is naar de moderators gestuurd.
report_duplicate = Je hebt dit al gemeld. De moderators gaan ernaar kijken.
report_spam = Spam
report_spam_body = Ongewenste reclame, oplichting, herhalende of misleidende inhoud
report_harassment = Intimidatie
report_harassment_body = Iemand lastigvallen, pesten of bedreigen
report_hate = Haat
report_hate_body = Valt mensen aan om wie ze zijn
report_violence = Geweld of zelfbeschadiging
report_violence_body = Bedreigingen, geweld verheerlijken, zelfbeschadiging aanmoedigen
report_misinformation = Misinformatie
report_misinformation_body = Valse beweringen die schade kunnen veroorzaken
report_impersonation = Valse identiteit
report_impersonation_body = Zich voordoen als iemand anders
report_other = Iets anders
report_other_body = Overtreedt de regels op een andere manier
post_edit = Bericht bewerken
post_edit_history = Bewerkingsgeschiedenis
post_report = Bericht melden
post_hide_reply = Antwoord verbergen
post_unhide_reply = Antwoord tonen
post_share_menu = Delen…
post_reply_hidden = Verborgen antwoord
post_reply_hidden_toast = Antwoord verborgen
post_reply_unhidden_toast = Antwoord weer zichtbaar
sensitive_label = Gevoelige media
sensitive_show = Tonen
sensitive_hide = Gevoelige media verbergen
media_label =
    { $count ->
        [one] { $count } bijlage
       *[other] { $count } bijlagen
    }
media_video = Video
media_open = Afbeelding { $index } van { $count } openen
lightbox_title = Mediaweergave
lightbox_prev = Vorige
lightbox_next = Volgende
poll_vote = Stemmen
poll_your_vote = Jouw stem
poll_votes =
    { $count ->
        [one] { $compact } stem
       *[other] { $compact } stemmen
    }
poll_final = Eindresultaat
poll_multiple = meerdere keuzes
poll_left_minutes = nog { $n } min
poll_left_hours = nog { $n } u
poll_left_days = nog { $n } d
pull_refresh = Trek omlaag om te vernieuwen
messages_title = Berichten
messages_new = Nieuw bericht
messages_label = Gesprekken
messages_empty_title = Begin een gesprek
messages_empty_body = Berichten die je verstuurt en ontvangt verschijnen hier.
messages_you_prefix = Jij: { $text }
messages_unread = { $count } ongelezen
messages_new_search = Mensen zoeken
messages_new_results = Mensen
messages_new_none = Niemand gevonden.
chat_load_older = Eerdere berichten laden
chat_intro = Dit is het begin van je gesprek.
chat_label = Berichten met { $name }
chat_delete = Verwijderen
chat_seen = Gezien
chat_delete_title = Dit bericht verwijderen?
chat_delete_body = Het verdwijnt voor jullie allebei.
chat_doc_title = Chat met { $name }
chat_error_title = Kan dit gesprek niet openen
chat_placeholder = Schrijf een bericht
chat_input_label = Bericht
chat_send = Versturen
lists_title = Lijsten
lists_new = Nieuwe lijst
lists_edit = Lijst bewerken
lists_label = Je lijsten
lists_empty_title = Nog geen lijsten
lists_empty_body = Groepeer mensen in lijsten om hun berichten apart te lezen.
lists_members =
    { $count ->
        [one] { $compact } lid
       *[other] { $compact } leden
    }
lists_private = Privé
lists_name = Naam
lists_description = Beschrijving
lists_private_switch = Privélijst
lists_save = Opslaan
lists_picker_title = { $handle } toevoegen aan lijsten
lists_picker_none = Je hebt nog geen lijsten.
lists_delete = Lijst verwijderen
lists_delete_title = Deze lijst verwijderen?
lists_delete_body = “{ $name }” wordt verwijderd. De mensen in de lijst blijven ongewijzigd.
lists_by = Lijst van { $handle }
lists_tab_posts = Berichten
lists_tab_members = Leden
lists_tabs = Lijstonderdelen
lists_error_title = Kan deze lijst niet openen
lists_posts_label = Lijstberichten
lists_posts_empty_title = Hier is nog niets
lists_posts_empty_body = Berichten van de leden van deze lijst verschijnen hier.
lists_members_label = Lijstleden
lists_members_empty_title = Geen leden
lists_members_empty_body = Voeg mensen toe vanaf hun profiel.
lists_remove = Verwijderen
scheduled_title = Ingeplande berichten
scheduled_label = Ingeplande berichten
scheduled_status_pending = Ingepland
scheduled_status_published = Geplaatst
scheduled_status_failed = Mislukt
scheduled_cw = Inhoudswaarschuwing: { $text }
scheduled_poll = Peiling met { $count } keuzes, loopt { $hours } u
scheduled_change_time = Tijd wijzigen
scheduled_post_now = Nu plaatsen
scheduled_remove = Verwijderen
scheduled_window_error = Kies een tijd tussen een minuut en 90 dagen vanaf nu.
scheduled_when = Plaatsen op
scheduled_zone = Tijdzone: { $zone }
scheduled_save = Opslaan
scheduled_posted = Geplaatst.
scheduled_cancel_title = Dit ingeplande bericht annuleren?
scheduled_cancel_body = Het wordt niet geplaatst.
scheduled_cancel_confirm = Bericht annuleren
scheduled_empty_title = Niets ingepland
scheduled_empty_body = Berichten die je inplant wachten hier tot het zover is.
profile_message = Bericht
profile_add_to_lists = Aan lijsten toevoegen
media_alt = ALT
profile_report = { $handle } melden
chart_show_table = { $title } als tabel tonen
chart_col_day = Dag
chart_col_hour = Uur (UTC)
chart_empty = { $title }: nog niets om te tonen.
chart_bar_summary = { $title }: { $total } in totaal, de meeste op { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } in totaal, { $last } op { $label }.
chart_heat_summary = { $title }: { $total } in totaal, drukst op { $label } om { $hour } ({ $peak }).
chart_heat_less = Minder
chart_heat_more = Meer
announce_region = Mededelingen
announce_dismiss = Mededeling sluiten
announce_more = Meer lezen
announce_tone_info = Info
announce_tone_success = Goed nieuws
announce_tone_warning = Waarschuwing
admin_stat_people = Mensen
admin_stat_people_sub = { $count } deze week actief
admin_stat_posts_today = Berichten, 24 u
admin_stat_posts_total = { $count } in totaal
admin_stat_reports = Openstaande meldingen
admin_stat_reports_open = Te beoordelen
admin_stat_reports_clear = Alles in orde
admin_stat_signups_today = Registraties vandaag
admin_stat_signups_sub = { $count } in 14 dagen
admin_stat_suspended = Opgeschort
admin_stat_suspended_sub = accounts
admin_chart_posts = Berichten per dag
admin_chart_signups = Registraties per dag
admin_chart_reports = Meldingen per dag
admin_chart_heat = Activiteit per weekdag en uur
nav_admin = Beheer
admin_title = Beheer
admin_sections = Beheeronderdelen
admin_tab_overview = Overzicht
admin_tab_reports = Meldingen
admin_tab_users = Mensen
admin_tab_instance = Instance
admin_tab_federation = Federatie
admin_tab_announcements = Mededelingen
admin_tab_invites = Uitnodigingen
admin_tab_filter = Inhoudsfilter
admin_tab_audit = Auditlogboek
admin_denied_title = Alleen voor medewerkers
admin_denied_body = Dit gedeelte is voor de beheerders en moderators van deze instance.
admin_denied_home = Naar home
admin_unknown = onbekend
admin_tag_active = Actief
admin_tag_suspended = Opgeschort
admin_tag_deleted = Verwijderd
admin_tag_org = Organisatie
admin_audit_system = systeem
admin_audit_empty = Nog niets vastgelegd
admin_report_status = Status van melding
admin_report_open = Open
admin_report_resolved = Afgehandeld
admin_reports_empty_open = Geen openstaande meldingen
admin_reports_empty_open_body = Wanneer mensen berichten of accounts melden, verschijnen ze hier.
admin_reports_empty_resolved = Nog niets afgehandeld
admin_report_by = gemeld door { $handle }
admin_report_same_target = { $count } openstaande meldingen
admin_report_media =
    { $count ->
        [one] { $count } bijlage
       *[other] { $count } bijlagen
    }
admin_report_poll = Bevat een peiling
admin_report_dismiss = Afwijzen
admin_report_delete_post = Bericht verwijderen
admin_report_suspend = Account opschorten
admin_report_delete_suspend = Verwijderen en opschorten
admin_report_confirm_title = Actie uitvoeren?
admin_report_confirm_body = { $action }. Dit wordt vastgelegd in het auditlogboek.
admin_report_confirm = Bevestigen
admin_report_resolution = { $action } door { $handle } · { $time }
admin_res_dismiss = Afgewezen
admin_res_delete_post = Bericht verwijderd
admin_res_suspend_user = Account opgeschort
admin_res_delete_and_suspend = Bericht verwijderd en account opgeschort
admin_users_search = Mensen zoeken
admin_users_search_placeholder = Naam of gebruikersnaam
admin_users_show = Tonen
admin_users_empty = Geen mensen gevonden
admin_filter_local = Lokaal
admin_filter_remote = Extern
admin_filter_staff = Medewerkers
admin_filter_verified = Geverifieerd
admin_filter_suspended = Opgeschort
admin_filter_all = Iedereen
admin_user_details = Details van { $handle }
admin_user_menu = { $handle } beheren
admin_user_posts =
    { $count ->
        [one] { $compact } bericht
       *[other] { $compact } berichten
    }
admin_user_followers =
    { $count ->
        [one] { $compact } volger
       *[other] { $compact } volgers
    }
admin_user_joined = lid sinds { $time }
admin_user_active = actief { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } openstaande melding
       *[other] { $compact } openstaande meldingen
    }
admin_act_verify = Verifiëren
admin_act_unverify = Verificatie intrekken
admin_act_org = Tot organisatie maken
admin_act_unorg = Organisatiestatus intrekken
admin_act_role_user = Medewerkersrol intrekken
admin_act_role_moderator = Tot moderator maken
admin_act_role_admin = Tot beheerder maken
admin_act_suspend = Opschorten
admin_act_unsuspend = Opschorting opheffen
admin_act_purge = Alle berichten verwijderen
admin_act_rename = Gebruikersnaam wijzigen
admin_act_reset_name = Weergavenaam herstellen
admin_act_clear_bio = Bio wissen
admin_act_delete = Account verwijderen
admin_done_verified = { $handle } is geverifieerd
admin_done_unverified = Verificatie ingetrokken
admin_done_org = { $handle } is nu een organisatie
admin_done_unorg = Organisatiestatus ingetrokken
admin_done_role = Rol bijgewerkt
admin_done_suspended = { $handle } opgeschort
admin_done_unsuspended = Opschorting van { $handle } opgeheven
admin_done_purged =
    { $count ->
        [one] { $count } bericht verwijderd
       *[other] { $count } berichten verwijderd
    }
admin_done_name_reset = Weergavenaam hersteld
admin_done_bio_cleared = Bio gewist
admin_done_deleted = { $handle } verwijderd
admin_done_renamed = Nu { $handle }
admin_role_phrase_user = een gewone gebruiker
admin_role_phrase_moderator = een moderator
admin_role_phrase_admin = een beheerder
admin_role_title = De rol van { $handle } wijzigen?
admin_role_body = Deze persoon wordt { $role }.
admin_role_confirm = Rol wijzigen
admin_unorg_title = Organisatiestatus van { $handle } intrekken?
admin_unorg_body = Alle aansluitingen worden beëindigd.
admin_unorg_confirm = Verwijderen
admin_suspend_title = { $handle } opschorten?
admin_suspend_body = Deze persoon wordt uitgelogd en de inhoud blijft verborgen totdat je de opschorting opheft.
admin_suspend_confirm = Opschorten
admin_purge_title = Elk bericht van { $handle } verwijderen?
admin_purge_body = Dit kan niet ongedaan worden gemaakt.
admin_purge_confirm = Alles verwijderen
admin_reset_name_title = De weergavenaam van { $handle } herstellen?
admin_reset_name_body = Die wordt vervangen door de gebruikersnaam.
admin_reset_name_confirm = Herstellen
admin_delete_title = { $handle } verwijderen?
admin_delete_body = Berichten, volgrelaties en gesprekken worden verwijderd en het account wordt geanonimiseerd. Dit kan niet ongedaan worden gemaakt.
admin_delete_confirm = Account verwijderen
admin_rename_title = De gebruikersnaam van { $handle } wijzigen
admin_rename_note = Links naar de oude gebruikersnaam werken niet meer.
admin_rename_handle = Nieuwe gebruikersnaam
admin_rename_reset_name = Ook de weergavenaam herstellen
admin_rename_save = Gebruikersnaam wijzigen
admin_detail_title = Accountgegevens
admin_detail_invite = Uitnodiging { $code }
admin_detail_posts = Berichten
admin_detail_followers = Volgers
admin_detail_following = Volgend
admin_detail_open_reports = Openstaande meldingen
admin_detail_sessions = Sessies
admin_detail_passkeys = Toegangssleutels
admin_detail_tokens = API-tokens
admin_detail_joined = Lid sinds { $time }
admin_detail_recent_posts = Recente berichten
admin_detail_no_posts = Geen berichten.
admin_detail_reports = Meldingen over dit account
admin_detail_no_reports = Geen meldingen.
admin_detail_history = Moderatiegeschiedenis
admin_detail_nothing = Nog niets.
admin_instance_title = Instance
admin_instance_name = Naam van de instance
admin_instance_description = Beschrijving
admin_instance_contact = Contact-e-mailadres
admin_instance_contact_hint = Getoond op de pagina’s Voorwaarden en Privacy
admin_instance_min_age = Minimumleeftijd
admin_instance_rules = Regels
admin_instance_rules_hint = Eén per regel
admin_instance_preview = Voorwaarden bekijken
admin_instance_save = Opslaan
admin_instance_saved = Instance-instellingen opgeslagen
admin_instance_template_note = De gebruiksvoorwaarden en het privacybeleid zijn sjablonen die deze waarden gebruiken. Laat ze beoordelen voor jouw rechtsgebied voordat je de instance openstelt voor het publiek.
admin_signups_title = Registraties
admin_signups_desc = Wie een account mag aanmaken op deze instance
admin_signups_open = Open
admin_signups_open_hint = Iedereen kan een account aanmaken.
admin_signups_invite = Alleen op uitnodiging
admin_signups_invite_hint = Mensen hebben een uitnodigingscode uit het tabblad Uitnodigingen nodig.
admin_signups_closed = Gesloten
admin_signups_closed_hint = Niemand kan zich registreren.
admin_filter_title = Inhoudsfilter
admin_filter_desc = Er wordt niets geblokkeerd totdat je hieronder woorden toevoegt. Geblokkeerde woorden gelden voor gebruikersnamen, weergavenamen, locaties, bio’s, berichten en lijstnamen, ook bij verhulde spelling. Gereserveerde gebruikersnamen zoals “admin” kunnen niet worden geregistreerd.
admin_filter_blocked = Geblokkeerde woorden
admin_filter_blocked_hint = Eén per regel, ook binnen woorden gevonden
admin_filter_allowed = Toegestane woorden
admin_filter_allowed_hint = Uitzonderingen op de geblokkeerde woorden, één per regel
admin_filter_reserved = Extra gereserveerde gebruikersnamen
admin_filter_reserved_hint = Eén per regel
admin_filter_save = Filter opslaan
admin_filter_saved = Filter opgeslagen
admin_filter_field_handle = gebruikersnaam
admin_filter_field_name = naam
admin_filter_field_location = locatie
admin_filter_field_bio = bio
admin_scan_title = Accounts scannen
admin_scan_desc = Zoek accounts waarvan de gebruikersnaam, naam of bio het filter overtreedt.
admin_scan_run = Nu scannen
admin_scan_clean = Geen accounts overtreden het filter.
admin_scan_found =
    { $count ->
        [one] { $count } account vraagt aandacht
       *[other] { $count } accounts vragen aandacht
    }
admin_scan_reason = De { $field } bevat { $reason }
admin_ann_title = Nieuwe mededeling
admin_ann_desc = Een banner bovenaan het scherm van iedereen totdat deze verloopt of je hem beëindigt. Mensen kunnen hem sluiten.
admin_ann_text = Tekst
admin_ann_link = Link
admin_ann_link_hint = Optioneel: https://… of /pad
admin_ann_tone = Stijl
admin_ann_tone_info = Info
admin_ann_tone_success = Goed nieuws
admin_ann_tone_warning = Waarschuwing
admin_ann_duration = Duur
admin_ann_days =
    { $count ->
        [one] { $count } dag
       *[other] { $count } dagen
    }
admin_ann_week = 1 week
admin_ann_until_ended = Tot beëindiging
admin_ann_publish = Publiceren
admin_ann_published = Mededeling gepubliceerd
admin_ann_list = Mededelingen tot nu toe
admin_ann_none = Nog geen mededelingen.
admin_ann_end = Beëindigen
admin_ann_ended = beëindigd
admin_ann_expired = verlopen
admin_ann_inactive = Niet zichtbaar
admin_ann_until = tot { $date }
admin_inv_title = Uitnodigingscodes
admin_inv_desc = Laat specifieke mensen meedoen terwijl registratie alleen op uitnodiging kan.
admin_inv_note = Notitie (voor wie)
admin_inv_uses = Aantal keer
admin_inv_use_count =
    { $count ->
        [one] { $count } keer
       *[other] { $count } keer
    }
admin_inv_expiry = Verloopt
admin_inv_week = Over 1 week
admin_inv_days =
    { $count ->
        [one] Over { $count } dag
       *[other] Over { $count } dagen
    }
admin_inv_never = Nooit
admin_inv_create = Code maken
admin_inv_created = Code { $code } gemaakt
admin_inv_created_copied = Code { $code } gemaakt en gekopieerd
admin_inv_list = Codes
admin_inv_none = Nog geen uitnodigingscodes.
admin_inv_state_active = Actief
admin_inv_state_revoked = Ingetrokken
admin_inv_state_used = Opgebruikt
admin_inv_state_expired = Verlopen
admin_inv_used = { $uses } van { $max } gebruikt
admin_inv_expires = verloopt { $date }
admin_inv_copy = Code { $code } kopiëren
admin_inv_copied = Code gekopieerd
admin_inv_copy_failed = Kopiëren mislukt. De code is { $code }.
admin_inv_revoke = Intrekken
admin_inv_revoke_title = Deze uitnodiging intrekken?
admin_inv_revoke_body = Mensen die er al mee lid zijn geworden, houden hun account.
settings_devices = Apparaten en sessies
settings_devices_desc = Bekijk waar je bent ingelogd en log op afstand uit
settings_passkeys = Toegangssleutels
settings_passkeys_desc = Log in met je vingerafdruk, gezicht, schermvergrendeling of een beveiligingssleutel
settings_affiliation = Aansluiting
settings_affiliation_desc = Organisaties waar je bij hoort, of de accounts van je organisatie
settings_bots = Bots en API
settings_bots_desc = API-tokens en het label “Geautomatiseerd”
settings_saved = Opgeslagen
passkey_title = Je toegangssleutels
passkey_desc = Log in met je vingerafdruk, gezicht, schermvergrendeling of beveiligingssleutel in plaats van een wachtwoord. Toegangssleutels zijn niet te phishen en er wordt niets geheims op onze server opgeslagen.
passkey_none = Nog geen toegangssleutels.
passkey_add = Toegangssleutel toevoegen
passkey_added = Toegangssleutel “{ $name }” toegevoegd
passkey_default_name = Toegangssleutel
passkey_added_ago = toegevoegd { $time }
passkey_used_ago = laatst gebruikt { $time }
passkey_never_used = nooit gebruikt
passkey_rename = Hernoemen
passkey_rename_label = Toegangssleutel { $name } hernoemen
passkey_rename_title = Toegangssleutel hernoemen
passkey_name = Naam
passkey_rename_save = Opslaan
passkey_remove = Verwijderen
passkey_remove_label = Toegangssleutel { $name } verwijderen
passkey_remove_title = “{ $name }” verwijderen?
passkey_remove_body = Je kunt niet meer inloggen met deze toegangssleutel. Misschien wil je hem ook van je apparaat of wachtwoordmanager verwijderen.
passkey_removed = Toegangssleutel verwijderd
passkey_unsupported = Deze browser kan op dit adres geen toegangssleutels gebruiken.
passkey_needs_domain = Voor toegangssleutels moet deze site een domeinnaam hebben, geen IP-adres.
passkey_already_here = Dit apparaat heeft al een toegangssleutel voor dit account.
token_scope_read = Lezen
token_scope_write = Plaatsen en reageren
token_scope_messages = Directe berichten
token_scope_read_hint = Tijdlijnen, meldingen, profielen, zoeken en de realtimestroom
token_scope_write_hint = Plaatsen, verwijderen, likes, reposts, bladwijzers, volgen en uploads
token_scope_messages_hint = Directe berichten lezen en versturen
token_title = API-tokens
token_desc = Met tokens kunnen scripts en bots je account via de API gebruiken. Behandel ze als wachtwoorden: wie een token heeft, kan binnen de bevoegdheden ervan namens jou handelen.
token_none = Nog geen tokens.
token_name = Naam van het token
token_name_hint = Bijvoorbeeld: weerbot
token_scopes = Bevoegdheden
token_create = Token aanmaken
token_used_ago = laatst gebruikt { $time }
token_never_used = nooit gebruikt
token_revoke = Intrekken
token_revoke_label = Token { $name } intrekken
token_revoke_title = “{ $name }” intrekken?
token_revoke_body = Alles wat het gebruikt, stopt met werken.
token_revoked = Token ingetrokken
token_secret_title = Je nieuwe token
token_secret_warning = Kopieer het nu. Voor je veiligheid wordt het niet opnieuw getoond.
token_secret_label = Nieuw API-token
token_copy = Kopiëren
token_copied = Token gekopieerd
token_copy_failed = Kopiëren mislukt. Selecteer het token en kopieer het handmatig.
token_done = Klaar
bot_title = Geautomatiseerd account
bot_desc = Toont het label “Geautomatiseerd” naast de naam, zodat mensen weten dat een bot de berichten plaatst.
bot_switch = Dit is een geautomatiseerd account
bot_owner = Beheerd door
bot_owner_hint = De gebruikersnaam van de persoon of organisatie erachter. Optioneel.
bot_owner_save = Opslaan
bot_docs = Ontwikkelaarsdocumentatie
bot_docs_desc = Endpoints, realtime-events en voorbeeldbots
aff_yours_title = Je aansluiting
aff_yours_desc = Een organisatie kan je uitnodigen om haar badge naast je naam te tonen. Er verandert niets tot je accepteert, en je kunt de organisatie altijd weer verlaten.
aff_none = Je bent niet aangesloten bij een organisatie.
aff_invites_title = Uitnodigingen
aff_accept = Accepteren
aff_decline = Weigeren
aff_leave = Verlaten
aff_leave_title = Deze organisatie verlaten?
aff_leave_body = De badge verdwijnt van je profiel.
aff_remove_title = { $handle } verwijderen?
aff_remove_body = Je badge verdwijnt van hun profiel.
aff_remove = Verwijderen
aff_remove_label = { $handle } verwijderen
aff_ended = Aansluiting beëindigd
aff_declined = Uitnodiging geweigerd
aff_accepted = Aansluiting toegevoegd
aff_invited = Uitnodiging verstuurd
aff_org_title = Aangesloten accounts
aff_org_desc = Nodig mensen uit om je badge naast hun naam te tonen. Ze moeten het accepteren.
aff_handle = Gebruikersnaam
aff_label = Label
aff_label_hint = Optioneel, zoals “Engineer”
aff_invite = Uitnodiging versturen
aff_members_none = Nog geen aangesloten accounts.
aff_pending = Uitgenodigd
aff_not_org = Ben je een organisatie? Vraag een beheerder van deze instance om je account zo te markeren.
auth_passkey = Inloggen met een toegangssleutel
auth_or = of
auth_invite = Uitnodigingscode
auth_invite_hint = Deze instance neemt alleen nieuwe mensen op uitnodiging aan.
auth_invite_required = Voer de uitnodigingscode in die je hebt gekregen.
landing_developers = Ontwikkelaars
landing_invite = Registreren kan alleen op uitnodiging. Heb je een code? Maak je account aan.
dev_title = Ontwikkelaars
dev_hero = Bots en API
dev_hero_sub = Bouw bots, scripts en integraties voor { $name }
dev_intro_h = Bouwen voor { $name }
dev_intro_p = Alles wat de tuita-apps doen loopt via een kleine JSON-API, en bots en scripts kunnen die ook gebruiken. Je hebt een account op { $name } en een persoonlijk API-token nodig. Je hoeft geen app te registreren en er is geen OAuth-gedoe.
dev_intro_1 = Maak een account voor je bot, of gebruik je eigen account.
dev_intro_2 = Zet in Instellingen, Bots en API, “Dit is een geautomatiseerd account” aan, zodat mensen een label “Geautomatiseerd” zien, en geef aan wie het beheert.
dev_intro_3 = Maak een token met de bevoegdheden die je nodig hebt en houd het geheim.
dev_auth_h = Authenticatie
dev_auth_p = Stuur het token mee in de Authorization-header. Elk verzoek en elk antwoord is JSON (UTF-8), behalve uploads, die multipart zijn.
dev_auth_prefix = Een token is tuita_ gevolgd door 43 letters, cijfers, streepjes en underscores. Alleen een hash wordt opgeslagen, dus een kwijtgeraakt token kan niet opnieuw worden getoond: maak een nieuw token. Een token heeft een of meer van deze bevoegdheden:
dev_auth_refused = Met een token kun je geen wachtwoorden, sessies, tokens of toegangssleutels wijzigen, je account niet exporteren of verwijderen en de beheer-API niet gebruiken. Daarvoor is echt inloggen op de website nodig en je krijgt een 403 terug.
dev_auth_errors = Fouten komen terug als JSON met een foutmelding en een 4xx-status. 401 betekent dat het token onjuist of ingetrokken is, 403 dat een bevoegdheid ontbreekt of de route is geweigerd, en bij 429 krijg je een Retry-After-header.
dev_post_h = Je eerste bericht
dev_post_p = Upload eerst afbeeldingen en video met POST /api/v1/media (multipart) en voeg ze dan op id toe in mediaIds. Een antwoord stelt replyTo in, een citaat stelt quoteOf in.
dev_endpoints_h = Endpoints
dev_endpoints_intro = Paden tussen accolades zijn plaatshouders. Lijsten pagineren met ?before= ingesteld op de next-waarde van de vorige pagina; limit is 1 tot 40. De kolom Bevoegdheid toont welke bevoegdheid het token nodig heeft.
dev_col_method = Methode
dev_col_path = Pad
dev_col_scope = Bevoegdheid
dev_col_what = Wat het doet
dev_ep_me = Het account waartoe het token behoort
dev_ep_patch_me = Profiel en accountinstellingen wijzigen
dev_ep_bot = Het label “Geautomatiseerd” aan- of uitzetten en de persoon noemen die de bot beheert
dev_ep_following = De hometijdlijn van de mensen die je volgt
dev_ep_foryou = De tijdlijn Voor jou; paginering met ?cursor=
dev_ep_notifications = Meldingen (?filter=mentions voor antwoorden, vermeldingen en citaten)
dev_ep_post = Een bericht met zijn draad en antwoorden
dev_ep_user = Een profiel (gebruikersnaam@domein voor accounts op andere servers)
dev_ep_user_posts = Berichten van iemand (?tab=replies, media of likes; bij media ook ?kind=photo of video)
dev_ep_search = Zoek berichten, mensen of media, met operators zoals from: en has:media
dev_ep_trends = De tags die de afgelopen week het meest zijn gebruikt
dev_ep_stream = Realtime-gebeurtenissen (Server-Sent Events)
dev_ep_create = Een bericht schrijven
dev_ep_thread = 2 tot 25 berichten als draad schrijven
dev_ep_delete = Een van je berichten verwijderen
dev_ep_like = Een bericht liken (DELETE om ongedaan te maken). Ook /repost en /bookmark
dev_ep_follow = Iemand volgen (DELETE om te ontvolgen)
dev_ep_media = Een afbeelding of video uploaden als multipart; het antwoord bevat het id om toe te voegen
dev_ep_schedule = Een bericht voor later inplannen
dev_ep_report = Een bericht of account melden bij de moderators
dev_ep_conversations = Je gesprekken met directe berichten
dev_ep_message = Een direct bericht versturen; de body is JSON met een text-veld
dev_realtime_h = Realtime-gebeurtenissen
dev_realtime_p = GET /api/v1/stream is een Server-Sent Events-stream. Die stuurt een notification-gebeurtenis bij elke like, repost, volger, elk antwoord, elke vermelding of elk citaat, een message-gebeurtenis voor directe berichten en badges voor aantallen ongelezen items. Elke 25 seconden stuurt hij een commentaarregel en hij vraagt clients na 5 seconden opnieuw te proberen; maak opnieuw verbinding als de verbinding wegvalt.
dev_example_h = Voorbeeld: een bot die vermeldingen beantwoordt
dev_example_node = Node 22 of nieuwer, zonder pakketten:
dev_example_python = Hetzelfde in Python, met alleen requests:
dev_limits_h = Limieten
dev_limit_rate = Elk token mag 600 verzoeken per 10 minuten doen. Boven de limiet antwoordt de server met 429 en een Retry-After-header.
dev_limit_tokens = Een account kan maximaal 20 tokens hebben en maximaal 20 tokens per uur maken.
dev_limit_posting = Plaatsen, volgen, melden en de andere acties hebben per account hun eigen limieten.
dev_rules_h = Regels voor bots
dev_rule_1 = Zet het label Geautomatiseerd aan en geef aan wie de bot beheert.
dev_rule_2 = Stuur alleen antwoorden of berichten naar mensen die erom hebben gevraagd: ze hebben de bot eerst genoemd, gevolgd of een bericht gestuurd. Stuur geen ongevraagde vermeldingen of directe berichten.
dev_rule_3 = Houd het rustig: blijf ruim onder de bovenstaande limieten.
dev_rule_4 = Volg de voorwaarden en regels van de instance. Moderators kunnen bots net als elk account opschorten.
admin_act_log_admin_bootstrap = Eerste beheerder ingesteld
admin_act_log_announcement_create = Mededeling gepubliceerd
admin_act_log_announcement_end = Mededeling beëindigd
admin_act_log_instance_settings = Instance-instellingen gewijzigd
admin_act_log_invite_create = Uitnodiging gemaakt
admin_act_log_invite_revoke = Uitnodiging ingetrokken
admin_act_log_post_delete = Bericht verwijderd
admin_act_log_report_dismiss = Melding afgewezen
admin_act_log_user_delete = Account verwijderd
admin_act_log_user_org = Als organisatie gemarkeerd
admin_act_log_user_unorg = Organisatiestatus ingetrokken
admin_act_log_user_purge_posts = Alle berichten verwijderd
admin_act_log_user_rename = Account hernoemd
admin_act_log_user_role = Rol gewijzigd
admin_act_log_user_suspend = Account opgeschort
admin_act_log_user_unsuspend = Opschorting van account opgeheven
admin_act_log_user_unverify = Verificatie ingetrokken
admin_act_log_user_verify = Account geverifieerd
admin_act_log_domain_block = Instance geblokkeerd
admin_act_log_domain_unblock = Blokkade van instance opgeheven
admin_act_log_federation_retry = Nieuwe bezorgpoging aangevraagd
admin_fed_stat_domains = Bekende instances
admin_fed_stat_domains_sub = gezien door deze instance
admin_fed_stat_accounts = Externe accounts
admin_fed_stat_accounts_sub = van elders bekend
admin_fed_stat_posts = Externe berichten
admin_fed_stat_posts_sub = lokaal opgeslagen
admin_fed_stat_queue = In wachtrij
admin_fed_stat_queue_sub = wachtende bezorgingen
admin_fed_queue_title = Bezorgwachtrij
admin_fed_queue_desc = Ondertekende activiteiten die wachten om andere instances te bereiken. Mislukte worden met toenemende vertraging opnieuw geprobeerd en na 12 pogingen verwijderd.
admin_fed_queue_empty = Er wacht niets. Alles is bezorgd.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } bezorging wacht
       *[other] { $compact } bezorgingen wachten
    }
admin_fed_queue_due = { $count } nu aan de beurt
admin_fed_queue_oldest = Oudste in wachtrij { $date }
admin_fed_retry = Nu opnieuw proberen
admin_fed_retry_done =
    { $count ->
        [one] { $compact } bezorging wordt nu opnieuw geprobeerd.
       *[other] { $compact } bezorgingen worden nu opnieuw geprobeerd.
    }
admin_fed_failing = Falende instances
admin_fed_failing_none = Geen enkele instance faalt.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } bezorging
       *[other] { $compact } bezorgingen
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } poging
       *[other] { $compact } pogingen
    }
admin_fed_failing_next = volgende poging { $date }
admin_fed_failing_error = Laatste fout
admin_fed_failing_no_error = Geen fout vastgelegd
admin_fed_actor_title = Instance-actor
admin_fed_actor_desc = Andere instances controleren de verzoeken die deze instance ondertekent aan de hand van deze sleutel.
admin_fed_actor_id = Actor
admin_fed_actor_key = Sleutel-id
admin_fed_actor_fingerprint = SHA-256-vingerafdruk
admin_fed_actor_copy = Vingerafdruk kopiëren
admin_fed_actor_copied = Vingerafdruk gekopieerd
admin_fed_actor_copy_failed = Kopiëren mislukt. De vingerafdruk is { $fingerprint }
admin_fed_blocks_title = Geblokkeerde instances
admin_fed_blocks_desc = Dempen verbergt de accounts van een instance op openbare tijdlijnen en in zoekresultaten. Opschorten verbergt ze overal, negeert wat ze sturen en stopt bezorging aan hen.
admin_fed_domain = Domein
admin_fed_domain_hint = example.social, of een link naar een willekeurige pagina erop
admin_fed_severity = Ernst
admin_fed_silence = Dempen
admin_fed_suspend = Opschorten
admin_fed_reason = Reden
admin_fed_reason_hint = Optioneel, alleen zichtbaar voor medewerkers
admin_fed_block = Instance blokkeren
admin_fed_blocked = { $domain } is geblokkeerd
admin_fed_none = Er zijn geen instances geblokkeerd.
admin_fed_list = Blokkades
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } bekend account
       *[other] { $compact } bekende accounts
    }
admin_fed_row_no_reason = Geen reden opgegeven
admin_fed_row_changed = { $domain } staat nu op { $severity }
admin_fed_remove = Verwijderen
admin_fed_remove_label = Blokkade van { $domain } opheffen
admin_fed_remove_title = Blokkade van { $domain } opheffen?
admin_fed_remove_body = De accounts en berichten komen terug en de bezorging begint weer.
admin_fed_removed = { $domain } is niet meer geblokkeerd
remote_domain_title = Account op { $domain }
remote_view_original = Bekijken op { $domain }
visibility_unlisted = Niet vermeld
visibility_unlisted_hint = Niet getoond op openbare tijdlijnen
visibility_followers = Alleen volgers
visibility_followers_hint = Alleen volgers kunnen dit zien
visibility_direct = Direct
visibility_direct_hint = Alleen de genoemde mensen kunnen dit zien
follow_requested = Aangevraagd
follow_cancel_request = Volgverzoek annuleren
follow_requested_toast = Volgverzoek verstuurd naar { $handle }
follow_cancelled_toast = Volgverzoek geannuleerd
profile_locked = Keurt volgers goed
profile_locked_hint = Volgverzoeken moeten door dit account worden goedgekeurd.
profile_original = Profiel bekijken op { $domain }
profile_remote_note = Een kopie van { $domain }. Aantallen en berichten zijn wat deze instance heeft gezien.
remote_lookup_loading = { $handle } opzoeken op { $domain }…
remote_lookup_notfound_title = Dat account bestaat niet
remote_lookup_notfound_body = { $handle } bestaat niet, of { $domain } deelt het niet.
remote_lookup_failed_title = Kan { $handle } niet laden
explore_lookup = { $target } opzoeken…
explore_lookup_empty_body = Dat account of bericht is niet gevonden. De andere instance is misschien offline, of deelt het niet.
language_label = Taal
oauth_title = Een app autoriseren
oauth_loading = Verzoek controleren…
oauth_wants = Deze app wil je account { $handle } gebruiken.
oauth_name_warning = Iedereen kan elke appnaam kiezen. Keur alleen apps goed die je vertrouwt.
oauth_website = Website
oauth_allow = De app kan het volgende
oauth_scope_read = Je profiel, berichten, volgrelaties, lijsten, tijdlijnen en meldingen lezen, en directe berichten die niet end-to-end versleuteld zijn.
oauth_scope_write = Namens jou plaatsen, bewerken en verwijderen, liken, volgen, blokkeren, dempen en je profiel en instellingen wijzigen.
oauth_scope_messages = Je directe berichten lezen en versturen.
oauth_push_note = Pushmeldingen worden niet ondersteund. Een app die erom vraagt, krijgt ze niet.
oauth_redirect = Na je antwoord word je doorgestuurd naar
oauth_redirect_oob = De app krijgt geen doorverwijzing. Hier wordt een code getoond die je in de app kunt plakken.
oauth_approve = Autoriseren
oauth_deny = Weigeren
oauth_switch = Ander account gebruiken
oauth_sent_approved = Geautoriseerd. Je gaat terug naar { $host }. Gebeurt er niets, sluit dan dit tabblad.
oauth_sent_denied = Geweigerd. Je gaat terug naar { $host }. Gebeurt er niets, sluit dan dit tabblad.
oauth_denied = Geweigerd. Er is niets met de app gedeeld.
oauth_code_title = Autorisatiecode
oauth_code_body = Plak deze code in de app. Hij werkt één keer en verloopt na 10 minuten.
oauth_code_copy = Code kopiëren
oauth_code_copied = Code gekopieerd
oauth_code_copy_failed = Kopiëren mislukt. Selecteer de code en kopieer hem handmatig.
oauth_error_title = Dit verzoek kan niet worden geautoriseerd
oauth_error_note = Er is niets naar de app gestuurd.
oauth_error_invalid_request = Het verzoek is ongeldig, of vraagt om terug te keren naar een adres dat de app niet heeft geregistreerd. Ga terug naar de app en probeer het opnieuw.
oauth_error_invalid_client = Deze app is hier niet geregistreerd.
oauth_error_invalid_scope = De app vraagt toegang die tuita niet biedt, of meer dan waarvoor hij is geregistreerd.
oauth_error_unsupported_response_type = De app vraagt om een soort inloggen die tuita niet ondersteunt.
oauth_error_rate = Je hebt te veel verzoeken beantwoord. Wacht een paar minuten en probeer het opnieuw.
oauth_error_unreachable = Kan tuita niet bereiken. Controleer je verbinding en probeer het opnieuw.
oauth_home = Naar tuita
share_send_chat = Via chat versturen
share_download = Media downloaden
share_download_n = Media { $number } downloaden
share_chat_title = Bericht via chat versturen
share_chat_sent = Verstuurd naar { $handle }
hover_card_label = Profielkaart van { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Mediatype
profile_media_all = Alle media
profile_media_photos = Foto's
profile_media_videos = Video's
profile_empty_videos_title = Nog geen video's
profile_empty_videos_body = Berichten met video's verschijnen hier.
account_add_existing = Bestaand account toevoegen
rail_label = Zoeken en suggesties
rail_search_submit = Zoeken
rail_show_more = Meer tonen
rail_footer = Overige links
chat_new = Nieuwe chat
chat_search = Gesprekken zoeken
chat_search_none = Geen gesprekken gevonden voor je zoekopdracht.
chat_start_title = Gesprek starten
chat_start_body = Kies een gesprek uit de lijst of begin een nieuw gesprek.
explore_sections = Verkenningsonderdelen
explore_see_more = Meer tonen
history_title = Geschiedenis
history_tabs = Geschiedenisonderdelen
history_likes_label = Gelikete berichten
video_play = Afspelen
video_pause = Pauzeren
video_mute = Dempen
video_unmute = Dempen opheffen
video_volume = Geluidsvolume
video_seek = Zoeken in video
video_position = { $current } van { $total }
video_speed = Afspeelsnelheid
video_fullscreen = Volledig scherm
video_exit_fullscreen = Volledig scherm sluiten
video_pip = Beeld-in-beeld
video_exit_pip = Beeld-in-beeld sluiten
video_error = Deze video kan niet worden afgespeeld.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Paneel verbergen
lightbox_show_panel = Paneel tonen
lightbox_view_post = Bericht bekijken
lightbox_panel = Bericht en reacties
emotepick_tabs = Emoji-sets
emotepick_tab_emoji = Emoji’s
emotepick_tab_custom = Eigen
admin_emote_tab = Eigen emoji’s
admin_emote_title = Eigen emoji’s
admin_emote_desc = Emotes horen bij de hele instance. Iedereen kan ze gebruiken als :shortcode: in berichten, namen en privéberichten. Een shortcode kan niet worden hernoemd; verwijder de emote en upload hem opnieuw.
admin_emote_shortcode = Codenaam
admin_emote_shortcode_hint = 2 tot 32 letters, cijfers of underscores. Wordt getoond als :shortcode:.
admin_emote_invalid = Gebruik alleen letters, cijfers en underscores, 2 tot 32 tekens.
admin_emote_category = Categorie
admin_emote_category_hint = Optioneel. Groepeert emotes in de kiezer.
admin_emote_file = Emote-afbeelding
admin_emote_file_hint = PNG, JPEG, WebP of GIF, tot 256 KB. Sleep een bestand hierheen of kies er een.
admin_emote_file_type = Dat bestand is geen PNG-, JPEG-, WebP- of GIF-afbeelding.
admin_emote_file_size = Afbeelding is te groot (max. 256 KB).
admin_emote_choose_file = Afbeelding kiezen
admin_emote_change_file = Afbeelding wijzigen
admin_emote_upload = Emote uploaden
admin_emote_upload_failed = Uploaden van de emote is mislukt. Probeer het opnieuw.
admin_emote_uploaded = :{ $shortcode }: toegevoegd.
admin_emote_list = Emotes op deze instance
admin_emote_none = Nog geen eigen emotes.
admin_emote_no_category = Geen categorie
admin_emote_category_for = Categorie van :{ $shortcode }:
admin_emote_saved = Categorie opgeslagen.
admin_emote_delete = :{ $shortcode }: verwijderen
admin_emote_delete_title = Deze emote verwijderen?
admin_emote_delete_body = :{ $shortcode }: wordt nergens meer als afbeelding weergegeven maar als gewone tekst. Berichten die hem gebruikten behouden de tekst.
admin_emote_delete_confirm = Emote verwijderen
admin_emote_deleted = Emote verwijderd.
admin_act_log_emoji_create = Emoji toegevoegd
admin_act_log_emoji_update = Emoji gewijzigd
admin_act_log_emoji_delete = Emoji verwijderd
e2ee_settings = Versleutelde berichten
e2ee_settings_desc = Wachtwoordzin, dit apparaat en resetten
e2ee_status_label = Toestand
e2ee_status_none = Uit. Zet het aan om end-to-end versleutelde berichten te versturen en te lezen.
e2ee_status_locked = Vergrendeld op dit apparaat. Voer je wachtwoordzin in om je berichten hier te lezen.
e2ee_status_ready = Aan en ontgrendeld op dit apparaat.
e2ee_keys_error = Je versleutelingssleutels konden niet worden gecontroleerd.
e2ee_setup_title = Versleutelde berichten aanzetten
e2ee_setup_body = Berichten aan mensen op deze server worden in je browser versleuteld. De server slaat alleen tekst op die hij niet kan lezen. Met een wachtwoordzin ontgrendel je je berichten op een ander apparaat.
e2ee_setup_warning = Je wachtwoordzin kan niet worden hersteld. Als je hem vergeet, kun je de versleuteling resetten, maar je eerdere berichten zijn dan kwijt.
e2ee_setup_submit = Aanzetten
e2ee_setup_done = Versleutelde berichten staan aan
e2ee_unlock_title = Versleutelde berichten ontgrendelen
e2ee_unlock_body = Voer je wachtwoordzin in om versleutelde berichten op dit apparaat te lezen en te versturen.
e2ee_unlock_submit = Ontgrendelen
e2ee_unlock_wrong = Die wachtwoordzin is onjuist.
e2ee_unlock_done = Ontgrendeld
e2ee_passphrase = Wachtwoordzin
e2ee_passphrase_confirm = Bevestig wachtwoordzin
e2ee_passphrase_hint = Minstens 10 tekens.
e2ee_passphrase_short = Gebruik minstens 10 tekens.
e2ee_passphrase_mismatch = De wachtwoordzinnen komen niet overeen.
e2ee_failed = Er ging iets mis. Probeer het opnieuw.
e2ee_unsupported = Deze browser ondersteunt geen versleutelde berichten.
e2ee_forget = Vergeten op dit apparaat
e2ee_forget_hint = Verwijdert je sleutel en de contacten die je vertrouwt uit deze browser. Met je wachtwoordzin ontgrendel je ze weer.
e2ee_forget_done = Op dit apparaat vergeten
e2ee_reset = Versleuteling resetten
e2ee_reset_hint = Maakt een nieuwe sleutel. Berichten die je eerder hebt verstuurd en ontvangen worden onleesbaar.
e2ee_reset_title = Versleuteling resetten?
e2ee_reset_body = Je eerdere berichten worden onleesbaar, voor jou en voor de mensen aan wie je schreef, en komen niet terug. Je contacten krijgen het verzoek je nieuwe sleutel te vertrouwen. Kies een nieuwe wachtwoordzin.
e2ee_reset_submit = Resetten
e2ee_reset_done = De versleuteling is gereset
e2ee_note_sealed = End-to-end versleuteld
e2ee_note_remote = Niet end-to-end versleuteld: deze persoon zit op een andere server.
e2ee_note_plain = Niet end-to-end versleuteld.
e2ee_plain_mark = Niet versleuteld
e2ee_preview = Versleuteld bericht
e2ee_line_sealed = Versleuteld bericht
e2ee_line_own_key = Verstuurd naar een sleutel die je niet meer hebt
e2ee_line_peer_key = Verstuurd met een sleutel die je nog niet vertrouwt
e2ee_line_unreadable = Kon niet worden geopend
e2ee_line_unsupported = Deze browser kan versleutelde berichten niet openen
e2ee_block_peer_off = { $handle } heeft versleutelde berichten nog niet aangezet.
e2ee_block_self_off = Zet eerst versleutelde berichten aan in Instellingen.
e2ee_block_unlock = Ontgrendel eerst versleutelde berichten in Instellingen.
e2ee_block_downgrade = { $handle } gebruikte eerder versleutelde berichten. Dit gesprek valt niet terug op onversleutelde berichten.
e2ee_block_changed = Vertrouw de nieuwe sleutel van { $handle } om te kunnen schrijven.
e2ee_block_error = Je versleutelingssleutels konden niet worden gecontroleerd.
e2ee_changed_title = De versleutelingssleutel van { $name } is veranderd
e2ee_changed_body = Misschien is de versleuteling gereset of is er een nieuw apparaat in gebruik, of zit er iemand tussen. Vergelijk het nieuwe veiligheidsnummer met die persoon voordat je het vertrouwt. Tot dan kun je niets versturen.
e2ee_changed_old = Veiligheidsnummer dat je vertrouwde
e2ee_changed_new = Nieuw veiligheidsnummer
e2ee_trust = Nieuwe sleutel vertrouwen
e2ee_safety_open = Veiligheidsnummer
e2ee_safety_title = Veiligheidsnummer
e2ee_safety_body = Vergelijk dit nummer met { $name } in persoon of tijdens een gesprek, niet in deze chat. Komt het op beide schermen overeen, dan zit er niemand tussen.

## 0031
composer_upload_failed = Uploaden mislukt. Probeer het opnieuw.
composer_send_failed = tuita is niet bereikbaar. Controleer je verbinding en probeer het opnieuw.
composer_open_failed = De berichteditor kon niet worden geopend. Probeer het opnieuw.
