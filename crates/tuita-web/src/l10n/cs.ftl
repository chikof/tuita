app_name = tuita
loading = Načítání
close = Zavřít
dismiss = Zavřít
notifications = Oznámení
avatar_more = Ještě { $count }
char_counter_left =
    { $count ->
        [one] Zbývá { $count } znak
        [few] Zbývají { $count } znaky
        [many] Zbývá { $count } znaku
       *[other] Zbývá { $count } znaků
    }
char_counter_over =
    { $count ->
        [one] Limit překročen o { $count } znak
        [few] Limit překročen o { $count } znaky
        [many] Limit překročen o { $count } znaku
       *[other] Limit překročen o { $count } znaků
    }
not_found_title = Tady nic není
not_found_body = Tato stránka neexistuje, nebo byla přesunuta.
not_found_back = Zpět
tagline = Každý příspěvek, dvě barvy.
error_title = Něco se pokazilo
cancel = Zrušit
retry = Zkusit znovu
back = Zpět
view = Zobrazit
follow = Sledovat
following = Sledovaní
follows_you = Sleduje vás
verified = Ověřený účet
bot_account = Automatizovaný účet
bot_label = Bot
link_copied = Odkaz zkopírován do schránky
text_copied = Text zkopírován
time_now = teď
time_minutes = { $n } min
time_hours = { $n } h
time_days = { $n } d
nav_main = Hlavní
nav_home = Domů
nav_explore = Prozkoumat
nav_notifications = Oznámení
nav_bookmarks = Záložky
nav_profile = Profil
nav_settings = Nastavení
nav_compose = Zveřejnit
nav_menu = Menu
badge_unread = Nepřečtené: { $count }
account_menu = Účty
account_switch = Přepnout účet
account_switch_to = Přepnout na { $handle }
account_add = Přidat účet
account_logout = Odhlásit { $handle }
account_logout_all = Odhlásit ze všech účtů
home_feed = Časová osa
home_for_you = Pro vás
home_following = Sledovaní
home_empty_title = Zatím tu nic není
home_empty_body = Buďte první, kdo něco zveřejní.
home_following_empty_title = Ve vašem okruhu je ticho
home_following_empty_body = Sledujte lidi a na této časové ose se zobrazí jejich příspěvky a sdílení.
home_find_people = Najít lidi
new_posts =
    { $count ->
        [one] Zobrazit { $count } nový příspěvek
        [few] Zobrazit { $count } nové příspěvky
        [many] Zobrazit { $count } nového příspěvku
       *[other] Zobrazit { $count } nových příspěvků
    }
new_posts_many = Zobrazit 99+ nových příspěvků
composer_title = Nový příspěvek
composer_reply_title = Odpovědět
composer_quote_title = Citovat příspěvek
composer_placeholder = Co se děje?
composer_reply_placeholder = Napište odpověď
composer_text_label = Text příspěvku
composer_replying_to = Odpověď uživateli { $handle }
composer_cw_label = Upozornění na obsah
composer_cw_placeholder = Upozornění na obsah, např. spoilery
composer_cw_toggle = Přidat upozornění na obsah
composer_sensitive_toggle = Označit média jako citlivá
composer_options = Možnosti příspěvku
composer_policy_label = Kdo může odpovídat
composer_suggestions = Návrhy
composer_tag_posts =
    { $count ->
        [one] { $compact } příspěvek
        [few] { $compact } příspěvky
        [many] { $compact } příspěvku
       *[other] { $compact } příspěvků
    }
composer_post = Zveřejnit
composer_reply = Odpovědět
policy_everyone = Odpovědět může kdokoli
policy_following = Odpovídat mohou lidé, které sledujete
policy_mentioned = Odpovídat mohou jen lidé, které zmíníte
post_sent = Příspěvek odeslán
post_by = Příspěvek od { $name }
post_reposted_by = { $name } sdílí
post_pinned = Připnutý příspěvek
post_replying_to = Odpověď uživateli { $handle }
post_replying_unavailable = Odpověď na nedostupný příspěvek
post_unavailable = Tento příspěvek není dostupný.
post_menu = Více
post_copy_link = Kopírovat odkaz
post_copy_text = Kopírovat text
post_pin = Připnout na profil
post_unpin = Odepnout z profilu
post_pinned_toast = Připnuto na váš profil
post_unpinned = Odepnuto z vašeho profilu
post_delete = Smazat příspěvek
post_delete_title = Smazat příspěvek?
post_delete_body = Nelze vrátit zpět. Příspěvek zmizí z vašeho profilu, z časových os vašich sledujících i z hledání.
post_deleted = Příspěvek smazán
post_view_profile = Zobrazit { $handle }
post_mute = Ztlumit { $handle }
post_block = Zablokovat { $handle }
post_reply_label = Odpovědi ({ $count })
post_reply_following_only = Odpovídat mohou jen lidé, které autor sleduje.
post_reply_mentioned_only = Odpovídat mohou jen lidé zmínění v tomto příspěvku.
post_repost_menu = Možnosti sdílení
post_repost = Sdílet znovu
post_undo_repost = Zrušit sdílení
post_quote = Citovat
post_reposts_label = Sdílení a citace ({ $count })
post_like_label = Lajky ({ $count })
post_bookmark_label = Přidat do záložek
post_share = Sdílet nebo kopírovat odkaz
cw_show = Zobrazit
cw_hide = Skrýt
cw_label = Upozornění na obsah: { $reason }
muted_toast = Ztlumeno: { $handle }
unmuted_toast = Ztlumení zrušeno: { $handle }
blocked_toast = Zablokováno: { $handle }
unblocked_toast = Odblokováno: { $handle }
block_title = Zablokovat { $handle }?
block_body = Tento účet vás nebude moci sledovat a vy neuvidíte jeho příspěvky. Nedozví se o tom.
block_confirm = Zablokovat
notify_like = @{ $handle } dal(a) lajk vašemu příspěvku
notify_repost = @{ $handle } sdílel(a) váš příspěvek
notify_reply = @{ $handle } vám odpověděl(a)
notify_mention = @{ $handle } vás zmínil(a)
notify_quote = @{ $handle } citoval(a) váš příspěvek
notify_follow = @{ $handle } vás začal(a) sledovat
notify_follow_request = @{ $handle } žádá o sledování
notify_poll = Anketa, ve které jste hlasovali, skončila
notify_affiliation = @{ $handle } vás pozval(a) k přidružení
shortcuts_title = Klávesové zkratky
shortcut_group_navigation = Navigace
shortcut_group_actions = Akce
shortcut_or = /
shortcut_then = potom
shortcut_plus = +
shortcut_next_prev = Další / předchozí příspěvek
shortcut_open = Otevřít příspěvek
shortcut_search = Hledat
shortcut_go_home = Domů
shortcut_go_explore = Prozkoumat
shortcut_go_notifications = Oznámení
shortcut_go_bookmarks = Záložky
shortcut_go_profile = Váš profil
shortcut_go_settings = Nastavení
shortcut_new_post = Nový příspěvek
shortcut_like = Dát lajk
shortcut_reply = Odpovědět
shortcut_repost = Sdílet nebo citovat
shortcut_bookmark = Přidat do záložek
shortcut_send = Odeslat příspěvek
shortcut_help = Zobrazit tuto nápovědu
auth_login_title = Přihlásit se
auth_add_title = Přidat další účet
auth_add_sub = Přihlaste se a přepínejte mezi až { $max } účty na tomto zařízení bez odhlašování.
auth_signup_title = Vytvořit účet
auth_handle = Uživatelské jméno
auth_password = Heslo
auth_display_name = Zobrazované jméno
auth_handle_hint = 3–15 písmen, číslic nebo podtržítek
auth_password_hint = Alespoň 8 znaků
auth_consent_pre = Je mi alespoň { $minAge } let a souhlasím s{" "}
auth_consent_terms = Podmínky služby
auth_consent_and = {" "}a{" "}
auth_consent_privacy = Zásady ochrany soukromí
auth_consent_end = {"."}
auth_consent_required = Přijměte prosím Podmínky služby a Zásady ochrany soukromí.
auth_signin = Přihlásit se
auth_signin_add = Přihlásit a přepnout
auth_create = Vytvořit účet
auth_to_signup = Jste tu poprvé? Vytvořte si účet
auth_to_login = Máte účet? Přihlaste se
auth_closed = Registrace jsou na této instanci momentálně uzavřeny.
landing_nav = Patička
landing_rules = Pravidla
landing_stats = Právě teď
landing_people = Lidé
landing_posts = Příspěvky
landing_clock = Váš místní čas
landing_trends = Populární
landing_trend =
    { $count ->
        [one] { $count } příspěvek
        [few] { $count } příspěvky
        [many] { $count } příspěvku
       *[other] { $count } příspěvků
    }
landing_join = Připojte se k { $name }
landing_join_sub = Vytvořte si účet a pište ve dvou barvách.
landing_closed = Registrace jsou momentálně uzavřeny. Už máte účet? Přihlaste se.
landing_terms = Podmínky
landing_privacy = Soukromí
landing_source = Zdrojový kód
landing_powered = Běží na tuita { $version }
docs_terms_title = Podmínky služby
docs_privacy_title = Zásady ochrany soukromí
docs_updated = Naposledy aktualizováno { $date }
docs_date = 5. listopadu 2026
docs_see_privacy = Viz také Zásady ochrany soukromí.
docs_see_terms = Viz také Podmínky služby.
docs_contact_fallback = správci instance (přes přímou zprávu nebo nahlášení)
docs_terms_agreement_h = 1. Souhlas
docs_terms_agreement_p = Tyto Podmínky služby („Podmínky“) upravují vaše používání služby { $name }, instance tuita na adrese { $domain } („služba“, „my“). Vytvořením účtu nebo používáním služby souhlasíte s těmito Podmínkami a se Zásadami ochrany soukromí. Pokud nesouhlasíte, službu nepoužívejte.
docs_terms_eligibility_h = 2. Kdo může službu používat
docs_terms_eligibility_p = Musí vám být alespoň { $minAge } let, případně musíte dosáhnout minimálního věku pro digitální souhlas v místě svého bydliště, pokud je vyšší. Musíte být způsobilí uzavřít závaznou smlouvu a nesmí vám být používání služby zakázáno platnými právními předpisy.
docs_terms_account_h = 3. Váš účet
docs_terms_account_1 = Své heslo udržujte v tajnosti. Za činnost na svém účtu odpovídáte vy.
docs_terms_account_2 = Uživatelské jméno se nesmí vydávat za jinou osobu či organizaci ani klamat ostatní o tom, kdo jste.
docs_terms_account_3 = Pokud se domníváte, že byl váš účet napaden, ihned nám to řekněte. Ostatní relace můžete odhlásit v Nastavení → Účet.
docs_terms_content_h = 4. Váš obsah
docs_terms_content_p1 = Vlastnictví toho, co zveřejníte, zůstává vám. Udělujete nám celosvětovou, nevýhradní licenci bez poplatků k ukládání, zobrazování, rozmnožování a šíření vašeho obsahu výhradně za účelem provozu a zlepšování služby. To zahrnuje odesílání na federované instance tuita (viz oddíl 6). Licence zaniká smazáním obsahu nebo účtu, s výjimkou kopií již doručených jiným instancím nebo uložených jinými lidmi.
docs_terms_content_p2 = Za to, co zveřejníte, odpovídáte. Zveřejňujte jen obsah, který máte právo sdílet.
docs_terms_rules_h = 5. Pravidla komunity
docs_terms_rules_lead = Používáním služby { $name } souhlasíte, že neporušíte zákon ani tato pravidla:
docs_terms_rules_p = Souhlasíte také, že na službu nebudete útočit, například obcházením bezpečnostních opatření, hromadným stahováním dat, šířením malwaru nebo přetěžováním.
docs_terms_federation_h = 6. Federace
docs_terms_federation_p = { $name } se může propojit („federovat“) s dalšími nezávislými instancemi tuita, které správci schválí. Když komunikujete s lidmi na jiné instanci, váš veřejný profil, veřejné příspěvky a interakce (například sledování, lajk, sdílení, odpověď nebo přímá zpráva) se na tuto instanci odešlou. Ostatní instance provozují jiní lidé podle vlastních podmínek. Obsah, který jiná instance získá, nemůžeme ovládat ani smazat, ale při smazání jí zasíláme žádosti o odstranění.
docs_terms_moderation_h = 7. Moderace a vymáhání
docs_terms_moderation_p = Moderátoři mohou odstranit obsah, omezit funkce nebo pozastavit účty, které porušují tyto Podmínky nebo zákon. Pokud to půjde, sdělíme vám důvod. Příspěvek nebo účet může nahlásit kdokoli. Nahlášení putují moderátorům této instance. S kteroukoli instancí můžeme kdykoli přestat federovat.
docs_terms_termination_h = 8. Odchod
docs_terms_termination_p = Data můžete kdykoli exportovat nebo účet smazat v Nastavení → Účet. Pokud tyto Podmínky vážně nebo opakovaně porušíte, můžeme váš přístup pozastavit nebo ukončit. Službu můžeme také zrušit a pokusíme se o tom předem přiměřeně informovat.
docs_terms_disclaimers_h = 9. Vyloučení záruk a odpovědnost
docs_terms_disclaimers_p = Služba je poskytována „tak, jak je“ a „jak je dostupná“, bez jakýchkoli záruk, v nejširším rozsahu, který zákon dovoluje. Neneseme odpovědnost za nepřímé, vedlejší ani následné škody ani za obsah zveřejněný uživateli či jinými instancemi. Nic v těchto Podmínkách neomezuje odpovědnost, kterou nelze ze zákona omezit, ani vaše zákonná práva spotřebitele.
docs_terms_changes_h = 10. Změny
docs_terms_changes_p = Tyto Podmínky můžeme aktualizovat. Pokud jde o významnou změnu, oznámíme ji v aplikaci před nabytím účinnosti. Pokračováním v používání služby poté nové Podmínky přijímáte.
docs_terms_contact_h = 11. Kontakt
docs_terms_contact_lead = Dotazy k těmto Podmínkám:
docs_privacy_summary_h = Stručně
docs_privacy_summary_1 = Žádné reklamy, žádné sledovací skripty, žádný prodej dat.
docs_privacy_summary_2 = Jedna cookie, díky které zůstanete přihlášeni. Předvolby vzhledu zůstávají ve vašem prohlížeči.
docs_privacy_summary_3 = Veřejné příspěvky jsou veřejné a mohou se dostat na federované instance tuita.
docs_privacy_summary_4 = Přímé zprávy mezi účty na tomto serveru jsou šifrované end-to-end, jakmile si to zapnou oba lidé: ukládají se jako text, který server nedokáže přečíst, takže únik databáze ani pohled do uložených dat nic neprozradí. Zprávy lidem na jiných serverech a zprávy odeslané před zapnutím šifrování nejsou šifrované tímto způsobem a správci je technicky mohou číst. Tento server také doručuje aplikaci, která šifruje, takže správce, který by tuto aplikaci změnil, by mohl zachytit, co píšete.
docs_privacy_summary_5 = Svá data můžete kdykoli exportovat nebo smazat.
docs_privacy_controller_h = 1. Kdo jsme
docs_privacy_controller_p = Tyto zásady se týkají služby { $name } na adrese { $domain }, kterou provozují správci instance. Ti rozhodují o tom, jak se vaše data zpracovávají.
docs_privacy_collect_h = 2. Co shromažďujeme
docs_privacy_collect_1 = Údaje o účtu: uživatelské jméno, zobrazované jméno, heslo (uložené jen jako hash argon2id se solí) a údaje v profilu: bio, poloha, web, avatar, banner, tvar a barva.
docs_privacy_collect_2 = Obsah: příspěvky, obrázky, odpovědi, lajky, sdílení, záložky, sledování, blokace, ztlumení a nahlášení.
docs_privacy_collect_3 = Přímé zprávy: ukládají se na tomto serveru, aby mohly být doručeny vám i druhé osobě. Zpráva šifrovaná end-to-end se ukládá jako šifrový text, který nemůžeme přečíst; kdo ji poslal, komu a kdy, nám zůstává vidět. Ostatní zprávy se ukládají jako prostý text. Váš šifrovací klíč se tu ukládá jen v podobě zamčené vaší heslovou frází, kterou nikdy nevidíme.
docs_privacy_collect_4 = Údaje o relacích: pro každé přihlášené zařízení user-agent prohlížeče, IP adresa a čas posledního použití, abyste relace mohli zkontrolovat a odhlásit.
docs_privacy_collect_5 = Krátkodobé bezpečnostní údaje: IP adresy držené v paměti několik minut kvůli omezování počtu požadavků.
docs_privacy_collect_p = Neshromažďujeme váš e-mail, telefonní číslo, kontakty ani polohu, pokud je sami nezadáte do profilu.
docs_privacy_use_h = 3. Jak údaje používáme
docs_privacy_use_p = Jen k provozu služby: zobrazení vašeho profilu a příspěvků, tvorbě časových os a oznámení, doručování zpráv, zabezpečení účtů, předcházení zneužití a vymáhání Podmínek. Časová osa „Pro vás“ řadí příspěvky podle lajků, sdílení, odpovědí a stáří. Nesestavuje o vás profil.
docs_privacy_visibility_h = 4. Co mohou vidět ostatní
docs_privacy_visibility_p = Váš profil, příspěvky, odpovědi, sdílení, lajky, sledující a sledovaní jsou veřejné pro všechny přihlášené na této instanci i pro federované instance. Když někdo sdílí odkaz na příspěvek nebo profil, aplikace jako Discord, Slack nebo iMessage mohou načíst náhled s veřejným příspěvkem, jeho obrázky a veřejnými údaji profilu. Příspěvky s upozorněním na obsah nebo označené jako citlivé se zobrazují v náhledu bez textu a obrázků. Záložky, blokace, ztlumení, nahlášení a nastavení jsou soukromé. Pokud vypnete „Zobrazovat mě v návrzích“, lidé vás stále najdou podle přesného uživatelského jména.
docs_privacy_federation_h = 5. Federace
docs_privacy_federation_p = Když někoho na jiné instanci tuita sledujete, odpovídáte mu, zmiňujete ho, dáváte lajk, sdílíte jeho příspěvek nebo mu píšete, odešleme této instanci potřebná data: váš veřejný profil, příspěvek či interakci a u přímých zpráv jejich text. Přímé zprávy na jiné instance nejsou šifrované end-to-end. Instance vašich sledujících dostávají vaše veřejné příspěvky a změny profilu. Tyto instance jsou provozovány nezávisle a ukládají data podle vlastních zásad. Když smažete příspěvek nebo účet, požádáme federované instance o smazání jejich kopií, ale nemůžeme zaručit, že to udělají.
docs_privacy_cookies_h = 6. Cookies a místní úložiště
docs_privacy_cookies_p = Používáme jednu nezbytně nutnou cookie relace s příznakem HttpOnly. Předvolby motivu, barvy, velikosti textu a pohybu jsou uloženy v místním úložišti vašeho prohlížeče a nikdy nám nejsou odeslány.
docs_privacy_retention_h = 7. Jak dlouho údaje uchováváme
docs_privacy_retention_1 = Obsah a údaje o účtu uchováváme, dokud je nesmažete.
docs_privacy_retention_2 = Smazané příspěvky okamžitě zmizí z veřejného zobrazení a jejich text i obrázky se vymažou.
docs_privacy_retention_3 = Když smažete účet, odstraníme vaše příspěvky, lajky, sledování, zprávy a relace a záznam účtu anonymizujeme.
docs_privacy_retention_4 = Relace vyprší po 30 dnech, nebo ihned po odhlášení.
docs_privacy_retention_5 = Moderační nahlášení a auditní záznam správy uchováváme tak dlouho, jak je nutné pro bezpečnost služby.
docs_privacy_rights_h = 8. Vaše práva
docs_privacy_rights_p = Podle toho, kde žijete (například podle GDPR nebo CCPA), můžete mít právo na přístup ke svým údajům, jejich opravu, export či smazání a právo vznést námitku proti zpracování nebo ho omezit. Většinu z toho zvládnete sami v Nastavení → Účet: upravit profil, stáhnout data a smazat účet. S čímkoli dalším se obraťte na:
docs_privacy_rights_end = Můžete také podat stížnost u místního úřadu na ochranu osobních údajů.
docs_privacy_security_h = 9. Zabezpečení
docs_privacy_security_p = Hesla se hashují pomocí scrypt. Tokeny relací ukládáme jen jako hashe. Požadavky mezi federovanými instancemi jsou podepsány klíči Ed25519. Žádný systém není dokonale bezpečný, proto používejte jedinečné heslo.
docs_privacy_children_h = 10. Děti
docs_privacy_children_p = Služba není určena dětem pod minimálním věkem uvedeným v Podmínkách. Pokud se domníváte, že účet má dítě, nahlaste to.
docs_privacy_changes_h = 11. Změny
docs_privacy_changes_p = Změny těchto zásad zveřejníme zde a aktualizujeme datum výše. Významné změny oznámíme v aplikaci.
docs_privacy_contact_h = 12. Kontakt
docs_privacy_contact_lead = Dotazy nebo žádosti týkající se soukromí:
profile_posts =
    { $count ->
        [one] { $compact } příspěvek
        [few] { $compact } příspěvky
        [many] { $compact } příspěvku
       *[other] { $compact } příspěvků
    }
profile_edit = Upravit profil
profile_more = Více
profile_copy_link = Kopírovat odkaz na profil
profile_mute = Ztlumit { $handle }
profile_unmute = Zrušit ztlumení { $handle }
profile_block = Zablokovat { $handle }
profile_unblock = Odblokovat { $handle }
profile_unblock_button = Odblokovat
profile_suspended = Tento účet je pozastaven. Vidí ho jen tým.
profile_muted = Ztlumeno
profile_org = Organizace
profile_role_admin = Správce
profile_role_moderator = Moderátor
profile_joined = Připojen(a) { $date }
profile_following_label = Sledovaní
profile_followers =
    { $count ->
        [one] Sledující
        [few] Sledující
        [many] Sledujícího
       *[other] Sledujících
    }
profile_counts = Počty sledování
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Sleduje: { $a }
            }
        [2]
            { $others ->
               *[other] Sledují: { $a } a { $b }
            }
       *[other]
            { $others ->
                [one] Sledují: { $a }, { $b } a { $others } další, kterého sledujete
                [few] Sledují: { $a }, { $b } a { $others } další, které sledujete
                [many] Sledují: { $a }, { $b } a { $others } dalšího, kterého sledujete
               *[other] Sledují: { $a }, { $b } a { $others } dalších, které sledujete
            }
    }
profile_banner = Banner profilu
profile_tabs = Části profilu
profile_tab_posts = Příspěvky
profile_tab_replies = Odpovědi
profile_tab_media = Média
profile_tab_likes = Lajky
profile_posts_label = Příspěvky
profile_empty_posts_self_title = Zatím jste nic nezveřejnili
profile_empty_posts_self_body = Zde se zobrazí vaše příspěvky a sdílení.
profile_empty_posts_title = { $handle } zatím nic nezveřejnil(a)
profile_empty_posts_body = Až něco zveřejní, objeví se to tady.
profile_empty_replies_title = Zatím žádné odpovědi
profile_empty_replies_body = Odpovědi se zobrazí tady.
profile_empty_media_title = Zatím žádné fotky
profile_empty_media_body = Zde se zobrazí příspěvky s obrázky.
profile_empty_likes_title = Zatím žádné lajky
profile_empty_likes_body = Zde se zobrazí příspěvky, které se vám líbí.
profile_blocked_title = Zablokovali jste { $handle }
profile_blocked_body = Neuvidíte jejich příspěvky a nemohou vás sledovat ani vám psát.
profile_hidden_title = Tyto příspěvky nevidíte
profile_hidden_body = { $handle } omezil(a), kdo může vidět profil.
follows_tabs = Seznamy sledování
follows_tab_followers = Sledující
follows_tab_following = Sledovaní
follows_label_followers = Sledující
follows_label_following = Sledovaní
follows_empty_followers_title = Zatím žádní sledující
follows_empty_followers_body = Až tento účet někdo začne sledovat, zobrazí se tady.
follows_empty_following_title = Nikoho nesleduje
follows_empty_following_body = Účty, které sleduje, se zobrazí tady.
thread_title = Příspěvek
thread_doc_title = { $name }: „{ $text }“
thread_views =
    { $count ->
        [one] Zobrazení
        [few] Zobrazení
        [many] Zobrazení
       *[other] Zobrazení
    }
thread_reposts =
    { $count ->
        [one] Sdílení
        [few] Sdílení
        [many] Sdílení
       *[other] Sdílení
    }
thread_quotes =
    { $count ->
        [one] Citace
        [few] Citace
        [many] Citace
       *[other] Citací
    }
thread_likes =
    { $count ->
        [one] Lajk
        [few] Lajky
        [many] Lajku
       *[other] Lajků
    }
thread_stats = Statistiky příspěvku
thread_replies = Odpovědi
thread_hidden = Skryté odpovědi ({ $count })
thread_hidden_note = Autor tyto odpovědi skryl.
thread_list_likes = Líbí se
thread_list_reposts = Sdíleli
thread_list_quotes = Citace
thread_list_empty = Zatím nikdo
tag_empty_title = Žádné příspěvky s { $tag }
tag_empty_body = Buďte první, kdo to použije.
tag_posts_label = Příspěvky s hashtagem { $tag }
bookmarks_title = Záložky
bookmarks_label = Příspěvky v záložkách
bookmarks_empty_title = Ukládejte si příspěvky na později
bookmarks_empty_body = Klepnutím na ikonu záložky si příspěvek uložíte sem. Záložky vidíte jen vy.
settings_account = Váš účet
settings_account_desc = Profil a heslo
settings_profile = Upravit profil
settings_profile_desc = Jméno, bio, tvar avataru a barva
settings_privacy = Soukromí a bezpečnost
settings_privacy_desc = Zprávy, dohledatelnost, ztlumená slova, blokované a ztlumené účty
settings_notifications = Oznámení
settings_notifications_desc = Vyberte, o čem dostávat upozornění
settings_appearance = Vzhled
settings_appearance_desc = Motiv, barva, velikost textu, kontrast a pohyb
settings_language = Jazyk
settings_language_desc = Jazyk rozhraní
settings_language_note = Uloženo v tomto prohlížeči a po přihlášení i ve vašem účtu.
settings_about = O této instanci
settings_about_desc = Pravidla, podmínky, soukromí, verze a klávesové zkratky
settings_logout = Odhlásit se
settings_account_profile = Profil
settings_account_joined = Připojen(a) { $date }
settings_account_edit = Upravit profil
settings_password_title = Změnit heslo
settings_password_current = Současné heslo
settings_password_new = Nové heslo
settings_password_hint = Alespoň 8 znaků. Ostatní zařízení budou odhlášena.
settings_password_confirm = Potvrďte nové heslo
settings_password_submit = Aktualizovat heslo
settings_password_mismatch = Nová hesla se neshodují.
settings_password_updated = Heslo aktualizováno. Ostatní relace byly odhlášeny.
settings_sessions_title = Zařízení a relace
settings_sessions_desc = Odhlaste vše, co nepoznáváte, a pak změňte heslo.
settings_device_on = { $browser } ({ $os })
settings_device_unknown = Neznámé zařízení
settings_session_current = Toto zařízení
settings_session_last_active = Naposledy aktivní: { $time }
settings_session_sign_out = Odhlásit
settings_session_sign_out_device = Odhlásit { $device }
settings_session_signed_out = Relace odhlášena
settings_sessions_revoke_all = Odhlásit ze všech ostatních zařízení
settings_sessions_revoke_title = Odhlásit ostatní zařízení?
settings_sessions_revoke_body = Všechna zařízení kromě tohoto budou odhlášena.
settings_sessions_revoke_confirm = Odhlásit ostatní
settings_sessions_revoked =
    { $count ->
        [one] Odhlášena { $count } relace
        [few] Odhlášeny { $count } relace
        [many] Odhlášeno { $count } relace
       *[other] Odhlášeno { $count } relací
    }
settings_profile_preview = Náhled
settings_profile_preview_alt = Náhled avataru
settings_profile_uploads = Nahrávání avataru a banneru přibude později. Do té doby se použije vygenerovaný avatar a banner.
settings_profile_details = Podrobnosti
settings_profile_name = Zobrazované jméno
settings_profile_bio = Bio
settings_profile_location = Poloha
settings_profile_website = Web
settings_profile_shape = Tvar avataru
settings_profile_ink = Barva profilu
settings_profile_save = Uložit profil
settings_profile_saved = Profil upraven
settings_shape_square = Čtverec
settings_shape_circle = Kruh
settings_shape_squircle = Zaoblený čtverec
settings_shape_octagon = Osmiúhelník
settings_ink_bone = Kostěná
settings_ink_phosphor = Fosforová
settings_ink_amber = Jantarová
settings_ink_cyan = Azurová
settings_ink_rose = Růžová
settings_dm_title = Přímé zprávy
settings_dm_desc = Kdo s vámi může začít konverzaci
settings_dm_label = Kdo vám může psát
settings_dm_everyone = Všichni
settings_dm_following = Lidé, které sleduji
settings_dm_none = Nikdo
settings_dm_saved = Nastavení zpráv uloženo
settings_discover_title = Dohledatelnost
settings_discoverable = Zobrazovat mě v návrzích a hledání
settings_discoverable_hint = Lidé vás stále najdou podle přesného uživatelského jména
settings_show_client = Zobrazovat, z jakého zařízení přispívám
settings_show_client_hint = Pod nové příspěvky přidá zařízení
settings_words_title = Ztlumená slova
settings_words_desc = Skrýt příspěvky s těmito slovy, frázemi či #hashtagy z časových os, hledání a odpovědí. Vaše příspěvky se nikdy neskryjí.
settings_words_label = Slovo ke ztlumení
settings_words_placeholder = Přidat slovo, frázi nebo #hashtag
settings_words_add = Ztlumit
settings_words_remove = Zrušit ztlumení: { $word }
settings_words_none = Žádná ztlumená slova.
settings_words_limit = Můžete ztlumit nejvýše { $count } slov.
settings_blocked_title = Blokované účty
settings_blocked_desc = Nemohou vás sledovat, psát vám, vidět vaše příspěvky ani na ně reagovat. Nedozví se o tom.
settings_blocked_empty = Nikoho jste nezablokovali.
settings_muted_title = Ztlumené účty
settings_muted_desc = Neuvidíte jejich příspěvky ani upozornění. Stále vás mohou sledovat a psát vám.
settings_muted_empty = Nikoho jste neztlumili.
settings_unblock = Odblokovat
settings_unmute = Zrušit ztlumení
settings_unblock_user = Odblokovat { $handle }
settings_unmute_user = Zrušit ztlumení { $handle }
settings_notif_title = Upozorňovat na
settings_notif_desc = Ztlumené a blokované účty vás nikdy neupozorní.
settings_notif_like = Lajky
settings_notif_like_hint = Když někdo dá lajk vašemu příspěvku
settings_notif_repost = Sdílení
settings_notif_repost_hint = Když někdo sdílí váš příspěvek
settings_notif_reply = Odpovědi
settings_notif_reply_hint = Když vám někdo odpoví
settings_notif_mention = Zmínky
settings_notif_mention_hint = Když vás někdo zmíní
settings_notif_quote = Citace
settings_notif_quote_hint = Když někdo cituje váš příspěvek
settings_notif_follow = Noví sledující
settings_notif_follow_hint = Když vás někdo začne sledovat
settings_desktop_title = Oznámení na ploše
settings_desktop_desc = Zobrazí se jako systémové oznámení, jen když je tato karta na pozadí.
settings_desktop_state = Oprávnění: { $state }
settings_desktop_allowed = Povoleno
settings_desktop_blocked = Zablokováno
settings_desktop_not_asked = Nepožádáno
settings_desktop_enable = Zapnout
settings_desktop_blocked_toast = Oznámení pro tento web jsou v nastavení prohlížeče blokována.
settings_desktop_unsupported = Tento prohlížeč nepodporuje oznámení na ploše.
settings_theme_title = Motiv
settings_appearance_note = Uloženo na tomto zařízení a použito hned.
settings_theme_mode = Motiv
settings_theme_system = Systém
settings_theme_light = Světlý
settings_theme_dark = Tmavý
settings_theme_ink = Barva
settings_theme_contrast = Kontrast
settings_contrast_normal = Běžný
settings_contrast_more = Vyšší
settings_reading_title = Text a pohyb
settings_theme_text_size = Velikost textu
settings_theme_motion = Pohyb
settings_motion_system = Systém
settings_motion_reduce = Omezit
settings_preview_title = Náhled
settings_preview_text = Takto se budou příspěvky číst s vaším nastavením.
settings_about_domain = Doména
settings_about_version = Verze
settings_about_contact = Kontakt
settings_about_terms = Podmínky služby
settings_about_privacy = Zásady ochrany soukromí
settings_about_source = Zdrojový kód
settings_about_rules = Pravidla
explore_search_label = Hledat příspěvky, lidi a hashtagy
explore_search_placeholder = Hledat příspěvky, lidi, #hashtagy
explore_clear = Vymazat hledání
explore_tips = Tipy pro hledání
explore_tips_title = Operátory hledání
explore_op_from = Příspěvky od účtu
explore_op_to = Odpovědi nebo zmínky adresované účtu
explore_op_media = Příspěvky s obrázky (nebo has:poll, has:links)
explore_op_since = Zveřejněno v dané datum nebo později
explore_op_until = Zveřejněno v dané datum nebo dříve
explore_op_phrase = Slova v tomto pořadí
explore_op_exclude = Vynechat příspěvky se slovem
explore_results = Výsledky hledání
explore_tab_posts = Příspěvky
explore_tab_people = Lidé
explore_tab_media = Média
explore_people = Lidé
explore_posts = Příspěvky
explore_recent = Nedávná hledání
explore_recent_clear = Vymazat
explore_trending = Populární
explore_trend_posts =
    { $count ->
        [one] { $compact } příspěvek
        [few] { $compact } příspěvky
        [many] { $compact } příspěvku
       *[other] { $compact } příspěvků
    }
explore_trends_empty = Zatím žádné trendy. Použijte #hashtag!
explore_suggestions = Koho sledovat
explore_suggestions_empty = Sledujete všechny. Působivé.
explore_no_posts_title = Žádné příspěvky pro „{ $q }“
explore_no_posts_body = Zkuste jiné slovo nebo hashtag.
explore_no_people_title = Žádní lidé pro „{ $q }“
explore_no_people_body = Zkuste jméno, uživatelské jméno nebo @jméno@instance pro někoho z jiné instance.
explore_no_media_title = Žádné obrázky pro „{ $q }“
explore_no_media_body = Zde se zobrazí odpovídající příspěvky s obrázky.
notif_filter = Filtr oznámení
notif_tab_all = Vše
notif_tab_mentions = Zmínky
notif_empty_all_title = Zatím žádná oznámení
notif_empty_all_body = Zde se zobrazí lajky, odpovědi, sdílení a noví sledující.
notif_empty_mentions_title = Zatím žádné zmínky
notif_empty_mentions_body = Zde se zobrazují odpovědi, zmínky a citace vašich příspěvků.
notif_who_two = { $first } a { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } a { $count } další
        [few] { $first }, { $second } a { $count } další
        [many] { $first }, { $second } a { $count } dalšího
       *[other] { $first }, { $second } a { $count } dalších
    }
notif_verb_like =
    { $count ->
        [one] dává lajk vašemu příspěvku
        [few] dávají lajk vašemu příspěvku
        [many] dávají lajk vašemu příspěvku
       *[other] dávají lajk vašemu příspěvku
    }
notif_verb_repost =
    { $count ->
        [one] sdílí váš příspěvek
        [few] sdílejí váš příspěvek
        [many] sdílejí váš příspěvek
       *[other] sdílejí váš příspěvek
    }
notif_verb_follow =
    { $count ->
        [one] vás začíná sledovat
        [few] vás začínají sledovat
        [many] vás začínají sledovat
       *[other] vás začínají sledovat
    }
notif_verb_follow_request =
    { $count ->
        [one] žádá o sledování
        [few] žádají o sledování
        [many] žádají o sledování
       *[other] žádají o sledování
    }
notif_verb_reply =
    { $count ->
        [one] vám odpovídá
        [few] vám odpovídají
        [many] vám odpovídají
       *[other] vám odpovídají
    }
notif_verb_mention =
    { $count ->
        [one] vás zmiňuje
        [few] vás zmiňují
        [many] vás zmiňují
       *[other] vás zmiňují
    }
notif_verb_quote =
    { $count ->
        [one] cituje váš příspěvek
        [few] citují váš příspěvek
        [many] citují váš příspěvek
       *[other] citují váš příspěvek
    }
notif_verb_poll =
    { $count ->
        [one] ukončil(a) anketu, ve které jste hlasovali
        [few] ukončili anketu, ve které jste hlasovali
        [many] ukončili anketu, ve které jste hlasovali
       *[other] ukončili anketu, ve které jste hlasovali
    }
notif_verb_affiliation =
    { $count ->
        [one] vás zve k přidružení
        [few] vás zvou k přidružení
        [many] vás zvou k přidružení
       *[other] vás zvou k přidružení
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Naplánovat
composer_post_all = Zveřejnit vše
composer_err_type = Tento soubor není obrázek ani video, které tuita umí použít (PNG, JPEG, GIF, WebP, MP4, WebM nebo MOV).
composer_err_image_size = Obrázek je příliš velký (max. 10 MB).
composer_err_video_size = Videa mohou mít až { $mb } MB.
composer_err_max_images = Můžete připojit nejvýše 4 obrázky.
composer_err_exclusive = Příspěvek může mít video, obrázky nebo anketu, ale jen jedno z toho.
composer_extra_placeholder = Přidat další příspěvek
composer_extra_label = Příspěvek ve vlákně { $n }
composer_extra_remove = Odebrat příspěvek ve vlákně { $n }
composer_schedule_chip = Zveřejní se { $time }
composer_schedule_clear = Neplánovat
composer_add_images = Přidat obrázky
composer_add_video = Přidat video
composer_add_poll = Přidat anketu
composer_add_post = Přidat další příspěvek
composer_scheduled = Naplánováno na { $time }
composer_thread_sent = Vlákno odesláno (příspěvků: { $count })
composer_remove_video = Odebrat video
composer_remove_image = Odebrat obrázek
composer_uploading = Nahrávání
composer_uploading_video = Nahrávání videa… { $percent } %
composer_alt_video = Popište toto video
composer_alt_image = Popište tento obrázek
composer_alt_label = Popis (alternativní text) přílohy { $index }
composer_emoji = Přidat emoji
poll_length_hours = { $n } h
poll_length_days = { $n } d
poll_choice = Možnost { $n }
poll_choice_optional = Možnost { $n } (volitelná)
poll_remove_choice = Odebrat možnost { $n }
poll_add_choice = Přidat možnost
poll_length = Délka ankety
poll_multiple_label = Povolit více možností
poll_remove = Odebrat anketu
schedule_title = Naplánovat příspěvek
schedule_hour = Za hodinu
schedule_evening = Dnes večer
schedule_tomorrow = Zítra ráno
schedule_week = Za týden
schedule_window = Vyberte čas od jedné minuty do 90 dní od teď.
schedule_custom = Vybrat datum a čas
schedule_set = Nastavit
schedule_clear = Neplánovat
schedule_view = Zobrazit naplánované příspěvky
dropzone_drop = Pusťte soubor sem
dropzone_reject = Tento soubor nelze použít
nav_messages = Zprávy
nav_lists = Seznamy
nav_scheduled = Naplánováno
notify_message = @{ $handle } vám poslal(a) zprávu
shortcut_go_messages = Zprávy
shortcut_go_lists = Seznamy
cropper_stage = Oblast oříznutí. Obrázek posunete tažením. Šipkami ho jemně posunete, plusem a mínusem změníte přiblížení.
cropper_hint = Přetažením posunete. Přiblížíte sevřením prstů, kolečkem nebo posuvníkem.
cropper_zoom = Přiblížení
cropper_zoom_in = Přiblížit
cropper_zoom_out = Oddálit
cropper_rotate = Otočit
cropper_reset = Obnovit
cropper_apply = Použít
image_field_avatar = Profilový obrázek
image_field_banner = Banner
image_field_edit_avatar = Upravit profilový obrázek
image_field_edit_banner = Upravit banner
image_field_change = Změnit
image_field_remove = Odebrat
image_field_hint = PNG, JPEG, GIF nebo WebP do 10 MB. GIF se změní na nepohyblivý obrázek.
image_field_not_image = Tento soubor není obrázek, který tuita umí použít (PNG, JPEG, GIF nebo WebP).
image_field_too_large = Obrázek je příliš velký (max. 10 MB).
image_field_uploading = Nahrávání
image_field_saved = Obrázek uložen.
image_field_removed = Obrázek odebrán.
edit_title = Upravit příspěvek
edit_note = Příspěvek můžete upravovat 30 minut po zveřejnění. Lidé vidí, že byl příspěvek upraven, i jeho dřívější verze.
edit_save = Uložit
edit_saved = Příspěvek upraven
edit_edited = Upraveno
edit_history_title = Historie úprav
edit_history_current = Aktuální verze
edit_history_earlier = Dřívější verze
edit_history_original = Původní
report_post_title = Nahlásit příspěvek
report_user_title = Nahlásit { $handle }
report_intro = Co je špatně? Moderátoři této instance vaše nahlášení posoudí. { $handle } neuvidí, kdo ho nahlásil.
report_reason = Důvod
report_note = Ještě něco? (volitelné)
report_send = Odeslat nahlášení
report_sent = Děkujeme. Nahlášení bylo odesláno moderátorům.
report_duplicate = Toto jste už nahlásili. Moderátoři se na to podívají.
report_spam = Spam
report_spam_body = Nevyžádaná reklama, podvody, opakující se nebo zavádějící obsah
report_harassment = Obtěžování
report_harassment_body = Napadání, šikanování nebo vyhrožování
report_hate = Nenávist
report_hate_body = Útočí na lidi kvůli tomu, kým jsou
report_violence = Násilí nebo sebepoškozování
report_violence_body = Výhrůžky, oslava násilí, podpora sebepoškozování
report_misinformation = Dezinformace
report_misinformation_body = Nepravdivá tvrzení, která mohou uškodit
report_impersonation = Vydávání se za jiného
report_impersonation_body = Vydává se za někoho jiného
report_other = Něco jiného
report_other_body = Porušuje pravidla jinak
post_edit = Upravit příspěvek
post_edit_history = Historie úprav
post_report = Nahlásit příspěvek
post_hide_reply = Skrýt odpověď
post_unhide_reply = Zobrazit odpověď
post_share_menu = Sdílet…
post_reply_hidden = Skrytá odpověď
post_reply_hidden_toast = Odpověď skryta
post_reply_unhidden_toast = Odpověď je znovu vidět
sensitive_label = Citlivá média
sensitive_show = Zobrazit
sensitive_hide = Skrýt citlivá média
media_label =
    { $count ->
        [one] { $count } příloha
        [few] { $count } přílohy
        [many] { $count } přílohy
       *[other] { $count } příloh
    }
media_video = Video
media_open = Otevřít obrázek { $index } z { $count }
lightbox_title = Prohlížeč médií
lightbox_prev = Předchozí
lightbox_next = Další
poll_vote = Hlasovat
poll_your_vote = Váš hlas
poll_votes =
    { $count ->
        [one] { $compact } hlas
        [few] { $compact } hlasy
        [many] { $compact } hlasu
       *[other] { $compact } hlasů
    }
poll_final = Konečné výsledky
poll_multiple = více možností
poll_left_minutes = Zbývá { $n } min
poll_left_hours = Zbývá { $n } h
poll_left_days = Zbývá { $n } d
pull_refresh = Potažením obnovíte
messages_title = Zprávy
messages_new = Nová zpráva
messages_label = Konverzace
messages_empty_title = Začněte konverzaci
messages_empty_body = Zprávy, které pošlete a obdržíte, se zobrazí tady.
messages_you_prefix = Vy: { $text }
messages_unread = Nepřečtené: { $count }
messages_new_search = Hledat lidi
messages_new_results = Lidé
messages_new_none = Nikdo nenalezen.
chat_load_older = Načíst dřívější zprávy
chat_intro = Toto je začátek vaší konverzace.
chat_label = Zprávy s { $name }
chat_delete = Smazat
chat_seen = Zobrazeno
chat_delete_title = Smazat tuto zprávu?
chat_delete_body = Zmizí vám oběma.
chat_doc_title = Chat s { $name }
chat_error_title = Konverzaci se nepodařilo otevřít
chat_placeholder = Napište zprávu
chat_input_label = Zpráva
chat_send = Odeslat
lists_title = Seznamy
lists_new = Nový seznam
lists_edit = Upravit seznam
lists_label = Vaše seznamy
lists_empty_title = Zatím žádné seznamy
lists_empty_body = Seskupte lidi do seznamů a čtěte jejich příspěvky zvlášť.
lists_members =
    { $count ->
        [one] { $compact } člen
        [few] { $compact } členové
        [many] { $compact } člena
       *[other] { $compact } členů
    }
lists_private = Soukromý
lists_name = Název
lists_description = Popis
lists_private_switch = Soukromý seznam
lists_save = Uložit
lists_picker_title = Přidat { $handle } do seznamů
lists_picker_none = Zatím nemáte žádné seznamy.
lists_delete = Smazat seznam
lists_delete_title = Smazat tento seznam?
lists_delete_body = Seznam „{ $name }“ bude odstraněn. Lidí v něm se to nijak nedotkne.
lists_by = Seznam od { $handle }
lists_tab_posts = Příspěvky
lists_tab_members = Členové
lists_tabs = Části seznamu
lists_error_title = Seznam se nepodařilo otevřít
lists_posts_label = Příspěvky ze seznamu
lists_posts_empty_title = Zatím tu nic není
lists_posts_empty_body = Zde se zobrazí příspěvky členů tohoto seznamu.
lists_members_label = Členové seznamu
lists_members_empty_title = Žádní členové
lists_members_empty_body = Přidejte lidi z jejich profilů.
lists_remove = Odebrat
scheduled_title = Naplánované příspěvky
scheduled_label = Naplánované příspěvky
scheduled_status_pending = Naplánováno
scheduled_status_published = Zveřejněno
scheduled_status_failed = Selhalo
scheduled_cw = Upozornění na obsah: { $text }
scheduled_poll = Anketa s počtem možností { $count }, trvá { $hours } h
scheduled_change_time = Změnit čas
scheduled_post_now = Zveřejnit teď
scheduled_remove = Odebrat
scheduled_window_error = Vyberte čas od jedné minuty do 90 dní od teď.
scheduled_when = Čas zveřejnění
scheduled_zone = Časové pásmo: { $zone }
scheduled_save = Uložit
scheduled_posted = Zveřejněno.
scheduled_cancel_title = Zrušit tento naplánovaný příspěvek?
scheduled_cancel_body = Nebude zveřejněn.
scheduled_cancel_confirm = Zrušit příspěvek
scheduled_empty_title = Nic není naplánováno
scheduled_empty_body = Příspěvky, které naplánujete, tu počkají do svého času.
profile_message = Zpráva
profile_add_to_lists = Přidat do seznamů
media_alt = ALT
profile_report = Nahlásit { $handle }
chart_show_table = Zobrazit { $title } jako tabulku
chart_col_day = Den
chart_col_hour = Hodina (UTC)
chart_empty = { $title }: zatím není co zobrazit.
chart_bar_summary = { $title }: celkem { $total }, nejvíce { $label } ({ $peak }).
chart_spark_summary = { $title }: celkem { $total }, { $label }: { $last }.
chart_heat_summary = { $title }: celkem { $total }, nejvíce { $label }, { $hour } ({ $peak }).
chart_heat_less = Méně
chart_heat_more = Více
announce_region = Oznámení
announce_dismiss = Zavřít oznámení
announce_more = Číst dál
announce_tone_info = Informace
announce_tone_success = Dobré zprávy
announce_tone_warning = Varování
admin_stat_people = Lidé
admin_stat_people_sub = Aktivní tento týden: { $count }
admin_stat_posts_today = Příspěvky, 24 h
admin_stat_posts_total = celkem { $count }
admin_stat_reports = Otevřená nahlášení
admin_stat_reports_open = Čeká na posouzení
admin_stat_reports_clear = Vše vyřízeno
admin_stat_signups_today = Dnešní registrace
admin_stat_signups_sub = { $count } za 14 dní
admin_stat_suspended = Pozastaveno
admin_stat_suspended_sub = účtů
admin_chart_posts = Příspěvky za den
admin_chart_signups = Registrace za den
admin_chart_reports = Nahlášení za den
admin_chart_heat = Aktivita podle dne v týdnu a hodiny
nav_admin = Správa
admin_title = Správa
admin_sections = Sekce správy
admin_tab_overview = Přehled
admin_tab_reports = Nahlášení
admin_tab_users = Lidé
admin_tab_instance = Instance
admin_tab_federation = Federace
admin_tab_announcements = Oznámení
admin_tab_invites = Pozvánky
admin_tab_filter = Filtr obsahu
admin_tab_audit = Auditní záznam
admin_denied_title = Pouze pro tým
admin_denied_body = Tato oblast je určena pro správce a moderátory této instance.
admin_denied_home = Domů
admin_unknown = neznámé
admin_tag_active = Aktivní
admin_tag_suspended = Pozastaven
admin_tag_deleted = Smazán
admin_tag_org = Organizace
admin_audit_system = systém
admin_audit_empty = Zatím nic nezaznamenáno
admin_report_status = Stav nahlášení
admin_report_open = Otevřená
admin_report_resolved = Vyřízená
admin_reports_empty_open = Žádná otevřená nahlášení
admin_reports_empty_open_body = Když lidé nahlásí příspěvky nebo účty, zobrazí se tady.
admin_reports_empty_resolved = Zatím nic nevyřízeno
admin_report_by = nahlásil(a) { $handle }
admin_report_same_target = Otevřených nahlášení: { $count }
admin_report_media =
    { $count ->
        [one] { $count } příloha
        [few] { $count } přílohy
        [many] { $count } přílohy
       *[other] { $count } příloh
    }
admin_report_poll = Obsahuje anketu
admin_report_dismiss = Zavřít
admin_report_delete_post = Smazat příspěvek
admin_report_suspend = Pozastavit účet
admin_report_delete_suspend = Smazat a pozastavit
admin_report_confirm_title = Provést akci?
admin_report_confirm_body = { $action }. Zaznamená se do auditního záznamu.
admin_report_confirm = Potvrdit
admin_report_resolution = { $action } · { $handle } · { $time }
admin_res_dismiss = Zamítnuto
admin_res_delete_post = Příspěvek smazán
admin_res_suspend_user = Účet pozastaven
admin_res_delete_and_suspend = Příspěvek smazán a účet pozastaven
admin_users_search = Hledat lidi
admin_users_search_placeholder = Jméno nebo uživatelské jméno
admin_users_show = Zobrazit
admin_users_empty = Nikdo nenalezen
admin_filter_local = Místní
admin_filter_remote = Vzdálení
admin_filter_staff = Tým
admin_filter_verified = Ověření
admin_filter_suspended = Pozastavení
admin_filter_all = Všichni
admin_user_details = Podrobnosti o { $handle }
admin_user_menu = Spravovat { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } příspěvek
        [few] { $compact } příspěvky
        [many] { $compact } příspěvku
       *[other] { $compact } příspěvků
    }
admin_user_followers =
    { $count ->
        [one] { $compact } sledující
        [few] { $compact } sledující
        [many] { $compact } sledujícího
       *[other] { $compact } sledujících
    }
admin_user_joined = připojen(a) { $time }
admin_user_active = aktivní { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } otevřené nahlášení
        [few] { $compact } otevřená nahlášení
        [many] { $compact } otevřeného nahlášení
       *[other] { $compact } otevřených nahlášení
    }
admin_act_verify = Ověřit
admin_act_unverify = Odebrat ověření
admin_act_org = Změnit na organizaci
admin_act_unorg = Odebrat status organizace
admin_act_role_user = Odebrat roli týmu
admin_act_role_moderator = Jmenovat moderátorem
admin_act_role_admin = Jmenovat správcem
admin_act_suspend = Pozastavit
admin_act_unsuspend = Obnovit
admin_act_purge = Smazat všechny příspěvky
admin_act_rename = Změnit uživatelské jméno
admin_act_reset_name = Obnovit zobrazované jméno
admin_act_clear_bio = Vymazat bio
admin_act_delete = Smazat účet
admin_done_verified = { $handle } je ověřen(a)
admin_done_unverified = Ověření odebráno
admin_done_org = { $handle } je nyní organizace
admin_done_unorg = Status organizace odebrán
admin_done_role = Role aktualizována
admin_done_suspended = { $handle } pozastaven(a)
admin_done_unsuspended = { $handle } obnoven(a)
admin_done_purged =
    { $count ->
        [one] Smazán { $count } příspěvek
        [few] Smazány { $count } příspěvky
        [many] Smazáno { $count } příspěvku
       *[other] Smazáno { $count } příspěvků
    }
admin_done_name_reset = Zobrazované jméno obnoveno
admin_done_bio_cleared = Bio vymazáno
admin_done_deleted = { $handle } smazán(a)
admin_done_renamed = Nově { $handle }
admin_role_phrase_user = běžný uživatel
admin_role_phrase_moderator = moderátor
admin_role_phrase_admin = správce
admin_role_title = Změnit roli uživatele { $handle }?
admin_role_body = Nová role: { $role }.
admin_role_confirm = Změnit roli
admin_unorg_title = Odebrat { $handle } status organizace?
admin_unorg_body = Všechna její přidružení skončí.
admin_unorg_confirm = Odebrat
admin_suspend_title = Pozastavit { $handle }?
admin_suspend_body = Účet bude odhlášen a jeho obsah zůstane skrytý, dokud ho neobnovíte.
admin_suspend_confirm = Pozastavit
admin_purge_title = Smazat všechny příspěvky uživatele { $handle }?
admin_purge_body = Nelze vrátit zpět.
admin_purge_confirm = Smazat vše
admin_reset_name_title = Obnovit zobrazované jméno uživatele { $handle }?
admin_reset_name_body = Nahradí se uživatelským jménem.
admin_reset_name_confirm = Obnovit
admin_delete_title = Smazat { $handle }?
admin_delete_body = Příspěvky, sledování a zprávy budou odstraněny a účet bude anonymizován. Nelze vrátit zpět.
admin_delete_confirm = Smazat účet
admin_rename_title = Změnit uživatelské jméno { $handle }
admin_rename_note = Odkazy na staré uživatelské jméno přestanou fungovat.
admin_rename_handle = Nové uživatelské jméno
admin_rename_reset_name = Obnovit také zobrazované jméno
admin_rename_save = Změnit uživatelské jméno
admin_detail_title = Podrobnosti o účtu
admin_detail_invite = Pozvánka { $code }
admin_detail_posts = Příspěvky
admin_detail_followers = Sledující
admin_detail_following = Sledovaní
admin_detail_open_reports = Otevřená nahlášení
admin_detail_sessions = Relace
admin_detail_passkeys = Přístupové klíče
admin_detail_tokens = Tokeny API
admin_detail_joined = Připojen(a) { $time }
admin_detail_recent_posts = Nedávné příspěvky
admin_detail_no_posts = Žádné příspěvky.
admin_detail_reports = Nahlášení tohoto účtu
admin_detail_no_reports = Žádná nahlášení.
admin_detail_history = Historie moderace
admin_detail_nothing = Zatím nic.
admin_instance_title = Instance
admin_instance_name = Název instance
admin_instance_description = Popis
admin_instance_contact = Kontaktní e-mail
admin_instance_contact_hint = Zobrazuje se na stránkách Podmínek a Soukromí
admin_instance_min_age = Minimální věk
admin_instance_rules = Pravidla
admin_instance_rules_hint = Jedno pravidlo na řádek
admin_instance_preview = Náhled podmínek
admin_instance_save = Uložit
admin_instance_saved = Nastavení instance uloženo
admin_instance_template_note = Podmínky služby a Zásady ochrany soukromí jsou šablony, které používají tyto hodnoty. Než instanci otevřete veřejnosti, nechte je zkontrolovat z hlediska vaší jurisdikce.
admin_signups_title = Registrace
admin_signups_desc = Kdo může na této instanci vytvořit účet
admin_signups_open = Otevřené
admin_signups_open_hint = Účet si může vytvořit kdokoli.
admin_signups_invite = Jen na pozvání
admin_signups_invite_hint = Lidé potřebují kód pozvánky ze záložky Pozvánky.
admin_signups_closed = Uzavřené
admin_signups_closed_hint = Nikdo se nemůže registrovat.
admin_filter_title = Filtr obsahu
admin_filter_desc = Nic není blokováno, dokud níže nepřidáte slova. Blokovaná slova platí pro uživatelská jména, zobrazovaná jména, polohy, bia, příspěvky a názvy seznamů, včetně maskovaného pravopisu. Rezervovaná uživatelská jména jako „admin“ nelze zaregistrovat.
admin_filter_blocked = Blokovaná slova
admin_filter_blocked_hint = Jedno na řádek, hledá se i uvnitř slov
admin_filter_allowed = Povolená slova
admin_filter_allowed_hint = Výjimky z blokovaných slov, jedno na řádek
admin_filter_reserved = Další rezervovaná uživatelská jména
admin_filter_reserved_hint = Jedno na řádek
admin_filter_save = Uložit filtr
admin_filter_saved = Filtr uložen
admin_filter_field_handle = uživatelské jméno
admin_filter_field_name = jméno
admin_filter_field_location = poloha
admin_filter_field_bio = bio
admin_scan_title = Zkontrolovat účty
admin_scan_desc = Najděte účty, jejichž uživatelské jméno, jméno nebo bio porušují filtr.
admin_scan_run = Zkontrolovat
admin_scan_clean = Žádný účet neporušuje filtr.
admin_scan_found =
    { $count ->
        [one] { $count } účet vyžaduje pozornost
        [few] { $count } účty vyžadují pozornost
        [many] { $count } účtu vyžaduje pozornost
       *[other] { $count } účtů vyžaduje pozornost
    }
admin_scan_reason = { $field }: obsahuje { $reason }
admin_ann_title = Nové oznámení
admin_ann_desc = Banner v horní části obrazovky všech, dokud nevyprší nebo ho neukončíte. Lidé ho mohou zavřít.
admin_ann_text = Zpráva
admin_ann_link = Odkaz
admin_ann_link_hint = Volitelné: https://… nebo /cesta
admin_ann_tone = Styl
admin_ann_tone_info = Informace
admin_ann_tone_success = Dobré zprávy
admin_ann_tone_warning = Varování
admin_ann_duration = Doba trvání
admin_ann_days =
    { $count ->
        [one] { $count } den
        [few] { $count } dny
        [many] { $count } dne
       *[other] { $count } dní
    }
admin_ann_week = 1 týden
admin_ann_until_ended = Do ukončení
admin_ann_publish = Zveřejnit
admin_ann_published = Oznámení zveřejněno
admin_ann_list = Dosavadní oznámení
admin_ann_none = Zatím žádná oznámení.
admin_ann_end = Ukončit
admin_ann_ended = ukončeno
admin_ann_expired = vypršelo
admin_ann_inactive = Nezobrazuje se
admin_ann_until = do { $date }
admin_inv_title = Kódy pozvánek
admin_inv_desc = Umožněte konkrétním lidem připojit se, když jsou registrace jen na pozvání.
admin_inv_note = Poznámka (pro koho je)
admin_inv_uses = Použití
admin_inv_use_count =
    { $count ->
        [one] { $count } použití
        [few] { $count } použití
        [many] { $count } použití
       *[other] { $count } použití
    }
admin_inv_expiry = Platnost
admin_inv_week = Za 1 týden
admin_inv_days =
    { $count ->
        [one] Za { $count } den
        [few] Za { $count } dny
        [many] Za { $count } dne
       *[other] Za { $count } dní
    }
admin_inv_never = Nikdy
admin_inv_create = Vytvořit kód
admin_inv_created = Kód { $code } vytvořen
admin_inv_created_copied = Kód { $code } vytvořen a zkopírován
admin_inv_list = Kódy
admin_inv_none = Zatím žádné kódy pozvánek.
admin_inv_state_active = Aktivní
admin_inv_state_revoked = Zrušený
admin_inv_state_used = Vyčerpaný
admin_inv_state_expired = Vypršel
admin_inv_used = Použito { $uses } z { $max }
admin_inv_expires = vyprší { $date }
admin_inv_copy = Kopírovat kód { $code }
admin_inv_copied = Kód zkopírován
admin_inv_copy_failed = Kopírování se nepodařilo. Kód je { $code }.
admin_inv_revoke = Zrušit
admin_inv_revoke_title = Zrušit tuto pozvánku?
admin_inv_revoke_body = Lidé, kteří se s ní už připojili, si účty ponechají.
settings_devices = Zařízení a relace
settings_devices_desc = Podívejte se, kde jste přihlášeni, a odhlaste se na dálku
settings_passkeys = Přístupové klíče
settings_passkeys_desc = Přihlaste se otiskem prstu, obličejem, zámkem obrazovky nebo bezpečnostním klíčem
settings_affiliation = Přidružení
settings_affiliation_desc = Organizace, do kterých patříte, nebo účty vaší organizace
settings_bots = Boti a API
settings_bots_desc = Tokeny API a štítek „Automatizovaný“
settings_saved = Uloženo
passkey_title = Vaše přístupové klíče
passkey_desc = Přihlaste se otiskem prstu, obličejem, zámkem obrazovky nebo bezpečnostním klíčem místo hesla. Přístupové klíče nelze phishingem získat a na našem serveru se neukládá nic tajného.
passkey_none = Zatím žádné přístupové klíče.
passkey_add = Přidat přístupový klíč
passkey_added = Přístupový klíč „{ $name }“ přidán
passkey_default_name = Přístupový klíč
passkey_added_ago = přidáno { $time }
passkey_used_ago = naposledy použito { $time }
passkey_never_used = nikdy nepoužito
passkey_rename = Přejmenovat
passkey_rename_label = Přejmenovat přístupový klíč { $name }
passkey_rename_title = Přejmenovat přístupový klíč
passkey_name = Název
passkey_rename_save = Uložit
passkey_remove = Odebrat
passkey_remove_label = Odebrat přístupový klíč { $name }
passkey_remove_title = Odebrat „{ $name }“?
passkey_remove_body = Pomocí tohoto přístupového klíče se už nebudete moci přihlásit. Možná ho budete chtít smazat i ze zařízení nebo správce hesel.
passkey_removed = Přístupový klíč odebrán
passkey_unsupported = Tento prohlížeč na této adrese nepodporuje přístupové klíče.
passkey_needs_domain = Přístupové klíče vyžadují, aby měl tento web doménové jméno, ne IP adresu.
passkey_already_here = Toto zařízení už má pro tento účet přístupový klíč.
token_scope_read = Čtení
token_scope_write = Přispívat a reagovat
token_scope_messages = Přímé zprávy
token_scope_read_hint = Časové osy, oznámení, profily, hledání a stream v reálném čase
token_scope_write_hint = Zveřejňování, mazání, lajky, sdílení, záložky, sledování a nahrávání
token_scope_messages_hint = Čtení a odesílání přímých zpráv
token_title = Tokeny API
token_desc = Tokeny umožňují skriptům a botům používat váš účet přes API. Chraňte je jako hesla: kdo má token, může v rámci jeho oprávnění jednat za vás.
token_none = Zatím žádné tokeny.
token_name = Název tokenu
token_name_hint = Například: bot s počasím
token_scopes = Oprávnění
token_create = Vytvořit token
token_used_ago = naposledy použito { $time }
token_never_used = nikdy nepoužito
token_revoke = Zrušit
token_revoke_label = Zrušit token { $name }
token_revoke_title = Zrušit „{ $name }“?
token_revoke_body = Vše, co ho používá, přestane fungovat.
token_revoked = Token zrušen
token_secret_title = Váš nový token
token_secret_warning = Zkopírujte ho teď. Z bezpečnostních důvodů se už nezobrazí.
token_secret_label = Nový token API
token_copy = Kopírovat
token_copied = Token zkopírován
token_copy_failed = Kopírování se nepodařilo. Vyberte token a zkopírujte ho ručně.
token_done = Hotovo
bot_title = Automatizovaný účet
bot_desc = U jména zobrazí štítek „Automatizovaný“, aby lidé věděli, že přispívá bot.
bot_switch = Toto je automatizovaný účet
bot_owner = Provozuje
bot_owner_hint = Uživatelské jméno osoby nebo organizace, která za ním stojí. Volitelné.
bot_owner_save = Uložit
bot_docs = Dokumentace pro vývojáře
bot_docs_desc = Endpointy, události v reálném čase a ukázkoví boti
aff_yours_title = Vaše přidružení
aff_yours_desc = Organizace vás může pozvat, abyste u jména zobrazovali její odznak. Dokud nepřijmete, nic se nezmění, a odejít můžete kdykoli.
aff_none = Nepatříte k žádné organizaci.
aff_invites_title = Pozvánky
aff_accept = Přijmout
aff_decline = Odmítnout
aff_leave = Odejít
aff_leave_title = Opustit tuto organizaci?
aff_leave_body = Její odznak zmizí z vašeho profilu.
aff_remove_title = Odebrat { $handle }?
aff_remove_body = Váš odznak z profilu tohoto účtu zmizí.
aff_remove = Odebrat
aff_remove_label = Odebrat { $handle }
aff_ended = Přidružení ukončeno
aff_declined = Pozvánka odmítnuta
aff_accepted = Přidružení přijato
aff_invited = Pozvánka odeslána
aff_org_title = Vaši přidružení
aff_org_desc = Pozvěte lidi, aby u svého jména zobrazovali váš odznak. Musí to přijmout.
aff_handle = Uživatelské jméno
aff_label = Štítek
aff_label_hint = Volitelné, například „Inženýr“
aff_invite = Odeslat pozvánku
aff_members_none = Zatím nikdo přidružený.
aff_pending = Pozván(a)
aff_not_org = Jste organizace? Požádejte správce této instance, aby váš účet označil.
auth_passkey = Přihlásit se přístupovým klíčem
auth_or = nebo
auth_invite = Kód pozvánky
auth_invite_hint = Tato instance přijímá nové lidi jen na pozvání.
auth_invite_required = Zadejte kód pozvánky, který jste dostali.
landing_developers = Vývojáři
landing_invite = Registrace je jen na pozvání. Máte kód? Vytvořte si účet.
dev_title = Vývojáři
dev_hero = Boti a API
dev_hero_sub = Tvořte boty, skripty a integrace pro { $name }
dev_intro_h = Tvořte pro { $name }
dev_intro_p = Vše, co aplikace tuita dělají, prochází malým JSON API a mohou ho používat i boti a skripty. Potřebujete účet na { $name } a osobní token API. Žádná registrace aplikace ani složitý OAuth.
dev_intro_1 = Vytvořte účet pro svého bota, nebo použijte vlastní.
dev_intro_2 = V Nastavení, Boti a API zapněte „Toto je automatizovaný účet“, aby lidé viděli štítek „Automatizovaný“, a uveďte, kdo ho provozuje.
dev_intro_3 = Vytvořte token s potřebnými oprávněními a udržujte ho v tajnosti.
dev_auth_h = Ověřování
dev_auth_p = Posílejte token v hlavičce Authorization. Každý požadavek i odpověď je JSON (UTF-8), kromě nahrávání, které je multipart.
dev_auth_prefix = Token je tuita_ následované 43 písmeny, číslicemi, pomlčkami a podtržítky. Ukládá se jen hash, takže ztracený token nelze znovu zobrazit: vytvořte nový. Token má jedno nebo více z těchto oprávnění:
dev_auth_refused = Token nemůže měnit hesla, relace, tokeny ani přístupové klíče, nemůže exportovat ani mazat účet a nemůže používat admin API. Na to je potřeba skutečné přihlášení na webu, jinak přijde odpověď 403.
dev_auth_errors = Chyby se vracejí jako JSON se zprávou error a stavem 4xx. 401 znamená chybný nebo zrušený token, 403 chybějící oprávnění nebo odmítnutou cestu a 429 přichází s hlavičkou Retry-After.
dev_post_h = Váš první příspěvek
dev_post_p = Nejprve nahrajte obrázky a video přes POST /api/v1/media (multipart), pak je připojte podle id v mediaIds. Odpověď nastavuje replyTo, citace nastavuje quoteOf.
dev_endpoints_h = Endpointy
dev_endpoints_intro = Cesty ve složených závorkách jsou zástupné hodnoty. Seznamy se stránkují pomocí ?before= nastaveného na další hodnotu z předchozí stránky; limit je 1 až 40. Sloupec Oprávnění uvádí oprávnění, které token potřebuje.
dev_col_method = Metoda
dev_col_path = Cesta
dev_col_scope = Oprávnění
dev_col_what = Co dělá
dev_ep_me = Účet, ke kterému token patří
dev_ep_patch_me = Změní profil a nastavení tohoto účtu
dev_ep_bot = Zapne nebo vypne štítek „Automatizovaný“ a uvede osobu, která bota provozuje
dev_ep_following = Domovská časová osa lidí, které sledujete
dev_ep_foryou = Časová osa Pro vás; stránkuje se pomocí ?cursor=
dev_ep_notifications = Oznámení (?filter=mentions pro odpovědi, zmínky a citace)
dev_ep_post = Příspěvek s vláknem a odpověďmi
dev_ep_user = Profil (handle@domain pro účty na jiných serverech)
dev_ep_user_posts = Příspěvky uživatele (?tab=replies, media nebo likes; u media také ?kind=photo nebo video)
dev_ep_search = Hledání příspěvků, lidí nebo médií s operátory jako from: a has:media
dev_ep_trends = Nejpoužívanější hashtagy za poslední týden
dev_ep_stream = Události v reálném čase (Server-Sent Events)
dev_ep_create = Napsat příspěvek
dev_ep_thread = Napsat 2 až 25 příspěvků jako vlákno
dev_ep_delete = Smazat jeden z vašich příspěvků
dev_ep_like = Dát příspěvku lajk (DELETE vrátí zpět). Také /repost a /bookmark
dev_ep_follow = Sledovat někoho (DELETE zruší sledování)
dev_ep_media = Nahrát obrázek nebo video jako multipart; odpověď obsahuje id k připojení
dev_ep_schedule = Naplánovat příspěvek na později
dev_ep_report = Nahlásit příspěvek nebo účet moderátorům
dev_ep_conversations = Vaše konverzace v přímých zprávách
dev_ep_message = Odeslat přímou zprávu; tělo je JSON s polem text
dev_realtime_h = Události v reálném čase
dev_realtime_p = GET /api/v1/stream je proud Server-Sent Events. Posílá událost notification pro každý lajk, sdílení, sledování, odpověď, zmínku nebo citaci, událost message pro přímé zprávy a badges s počty nepřečtených. Každých 25 sekund posílá řádek komentáře a žádá klienty o opakování po 5 sekundách; pokud se spojení přeruší, připojte se znovu.
dev_example_h = Příklad: bot, který odpovídá na zmínky
dev_example_node = Node 22 nebo novější, bez balíčků:
dev_example_python = Totéž v Pythonu, jen s requests:
dev_limits_h = Limity
dev_limit_rate = Každý token může provést 600 požadavků za 10 minut. Po překročení limitu server odpoví 429 s hlavičkou Retry-After.
dev_limit_tokens = Účet může mít až 20 tokenů a vytvořit až 20 tokenů za hodinu.
dev_limit_posting = Zveřejňování, sledování, nahlašování a další akce mají vlastní limity na účet.
dev_rules_h = Pravidla pro boty
dev_rule_1 = Zapněte štítek Automatizovaný a uveďte, kdo bota provozuje.
dev_rule_2 = Odpovídejte nebo pište jen lidem, kteří o to požádali: zmínili bota, sledují ho nebo mu napsali jako první. Neposílejte nevyžádané zmínky ani přímé zprávy.
dev_rule_3 = Buďte ohleduplní: držte se hluboko pod výše uvedenými limity.
dev_rule_4 = Dodržujte Podmínky a pravidla instance. Moderátoři mohou boty pozastavit jako každý účet.
admin_act_log_admin_bootstrap = Nastaven první správce
admin_act_log_announcement_create = Oznámení zveřejněno
admin_act_log_announcement_end = Oznámení ukončeno
admin_act_log_instance_settings = Nastavení instance změněno
admin_act_log_invite_create = Pozvánka vytvořena
admin_act_log_invite_revoke = Pozvánka zrušena
admin_act_log_post_delete = Příspěvek smazán
admin_act_log_report_dismiss = Nahlášení zamítnuto
admin_act_log_user_delete = Účet smazán
admin_act_log_user_org = Označen jako organizace
admin_act_log_user_unorg = Status organizace odebrán
admin_act_log_user_purge_posts = Všechny příspěvky smazány
admin_act_log_user_rename = Účet přejmenován
admin_act_log_user_role = Role změněna
admin_act_log_user_suspend = Účet pozastaven
admin_act_log_user_unsuspend = Účet obnoven
admin_act_log_user_unverify = Ověření odebráno
admin_act_log_user_verify = Účet ověřen
admin_act_log_domain_block = Instance zablokována
admin_act_log_domain_unblock = Instance odblokována
admin_act_log_federation_retry = Požádáno o opakování doručení
admin_fed_stat_domains = Známé instance
admin_fed_stat_domains_sub = viděné touto instancí
admin_fed_stat_accounts = Vzdálené účty
admin_fed_stat_accounts_sub = známé odjinud
admin_fed_stat_posts = Vzdálené příspěvky
admin_fed_stat_posts_sub = uložené lokálně
admin_fed_stat_queue = Ve frontě
admin_fed_stat_queue_sub = doručení čeká
admin_fed_queue_title = Fronta doručení
admin_fed_queue_desc = Podepsané aktivity čekající na doručení jiným instancím. Neúspěšné se opakují se stále delšími prodlevami a po 12 pokusech se zahodí.
admin_fed_queue_empty = Nic nečeká. Všechna doručení byla odeslána.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } doručení čeká
        [few] { $compact } doručení čekají
        [many] { $compact } doručení čeká
       *[other] { $compact } doručení čeká
    }
admin_fed_queue_due = K odeslání nyní: { $count }
admin_fed_queue_oldest = Nejstarší ve frontě od { $date }
admin_fed_retry = Zkusit znovu
admin_fed_retry_done =
    { $count ->
        [one] { $compact } doručení bude nyní zopakováno.
        [few] { $compact } doručení budou nyní zopakována.
        [many] { $compact } doručení bude nyní zopakováno.
       *[other] { $compact } doručení bude nyní zopakováno.
    }
admin_fed_failing = Instance s chybami
admin_fed_failing_none = Žádná instance nemá chyby.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } doručení
        [few] { $compact } doručení
        [many] { $compact } doručení
       *[other] { $compact } doručení
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } pokus
        [few] { $compact } pokusy
        [many] { $compact } pokusu
       *[other] { $compact } pokusů
    }
admin_fed_failing_next = další pokus { $date }
admin_fed_failing_error = Poslední chyba
admin_fed_failing_no_error = Žádná chyba nezaznamenána
admin_fed_actor_title = Aktér instance
admin_fed_actor_desc = Ostatní instance ověřují požadavky, které tato instance podepisuje, proti tomuto klíči.
admin_fed_actor_id = Aktér
admin_fed_actor_key = ID klíče
admin_fed_actor_fingerprint = Otisk SHA-256
admin_fed_actor_copy = Kopírovat otisk
admin_fed_actor_copied = Otisk zkopírován
admin_fed_actor_copy_failed = Kopírování se nepodařilo. Otisk je { $fingerprint }
admin_fed_blocks_title = Blokované instance
admin_fed_blocks_desc = Umlčení skryje účty instance z veřejných časových os a hledání. Pozastavení je skryje všude, zahodí vše, co pošlou, a zastaví doručování k nim.
admin_fed_domain = Doména
admin_fed_domain_hint = example.social, nebo odkaz na libovolnou stránku na ní
admin_fed_severity = Závažnost
admin_fed_silence = Umlčet
admin_fed_suspend = Pozastavit
admin_fed_reason = Důvod
admin_fed_reason_hint = Volitelné, vidí jen tým
admin_fed_block = Zablokovat instanci
admin_fed_blocked = { $domain } je zablokována
admin_fed_none = Žádné instance nejsou blokovány.
admin_fed_list = Blokace
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } známý účet
        [few] { $compact } známé účty
        [many] { $compact } známého účtu
       *[other] { $compact } známých účtů
    }
admin_fed_row_no_reason = Důvod neuveden
admin_fed_row_changed = { $domain } má nyní nastaveno: { $severity }
admin_fed_remove = Odebrat
admin_fed_remove_label = Zrušit blokaci { $domain }
admin_fed_remove_title = Zrušit blokaci { $domain }?
admin_fed_remove_body = Její účty a příspěvky se vrátí a doručování k ní se obnoví.
admin_fed_removed = { $domain } už není blokována
remote_domain_title = Účet na { $domain }
remote_view_original = Zobrazit na { $domain }
visibility_unlisted = Neuvedený
visibility_unlisted_hint = Nezobrazuje se na veřejných časových osách
visibility_followers = Jen sledující
visibility_followers_hint = Vidí to jen sledující
visibility_direct = Přímý
visibility_direct_hint = Vidí to jen zmínění lidé
follow_requested = Požádáno
follow_cancel_request = Zrušit žádost o sledování
follow_requested_toast = Žádost o sledování odeslána uživateli { $handle }
follow_cancelled_toast = Žádost o sledování zrušena
profile_locked = Schvaluje sledující
profile_locked_hint = Žádosti o sledování vyžadují schválení tohoto účtu.
profile_original = Zobrazit profil na { $domain }
profile_remote_note = Kopie z { $domain }. Počty a příspěvky odpovídají tomu, co tato instance viděla.
remote_lookup_loading = Hledání { $handle } na { $domain }…
remote_lookup_notfound_title = Takový účet neexistuje
remote_lookup_notfound_body = { $handle } neexistuje, nebo ho { $domain } nesdílí.
remote_lookup_failed_title = Nepodařilo se načíst { $handle }
explore_lookup = Hledání { $target }…
explore_lookup_empty_body = Účet ani příspěvek nebyl nalezen. Druhá instance může být offline, nebo ho nesdílí.
language_label = Jazyk
oauth_title = Autorizovat aplikaci
oauth_loading = Kontrola požadavku…
oauth_wants = Tato aplikace chce používat váš účet { $handle }.
oauth_name_warning = Název aplikace si může zvolit kdokoli. Schvalujte jen aplikace, kterým důvěřujete.
oauth_website = Web
oauth_allow = Bude moci
oauth_scope_read = Číst váš profil, příspěvky, sledování, seznamy, časové osy a oznámení a přímé zprávy, které nejsou šifrované end-to-end.
oauth_scope_write = Za vás zveřejňovat, upravovat a mazat příspěvky, dávat lajky, sledovat, blokovat, ztlumovat a měnit váš profil a nastavení.
oauth_scope_messages = Číst a odesílat vaše přímé zprávy.
oauth_push_note = Push oznámení nejsou podporována. Aplikace, která o ně požádá, je nedostane.
oauth_redirect = Po vaší odpovědi budete přesměrováni na
oauth_redirect_oob = Aplikace nedostane přesměrování. Zobrazí se tu kód, který do ní vložíte.
oauth_approve = Autorizovat
oauth_deny = Zamítnout
oauth_switch = Použít jiný účet
oauth_sent_approved = Autorizováno. Vracíme vás na { $host }. Pokud se nic nestane, zavřete tuto kartu.
oauth_sent_denied = Zamítnuto. Vracíme vás na { $host }. Pokud se nic nestane, zavřete tuto kartu.
oauth_denied = Zamítnuto. Aplikaci nebylo nic předáno.
oauth_code_title = Autorizační kód
oauth_code_body = Vložte tento kód do aplikace. Funguje jednou a vyprší za 10 minut.
oauth_code_copy = Kopírovat kód
oauth_code_copied = Kód zkopírován
oauth_code_copy_failed = Kopírování se nepodařilo. Vyberte kód a zkopírujte ho ručně.
oauth_error_title = Tento požadavek nelze autorizovat
oauth_error_note = Aplikaci nebylo nic odesláno.
oauth_error_invalid_request = Požadavek je chybný, nebo žádá o návrat na adresu, kterou aplikace neregistrovala. Vraťte se do aplikace a zkuste to znovu.
oauth_error_invalid_client = Tato aplikace tu není registrována.
oauth_error_invalid_scope = Aplikace žádá o přístup, který tuita nenabízí, nebo o víc, než zaregistrovala.
oauth_error_unsupported_response_type = Aplikace žádá o druh přihlášení, který tuita nepodporuje.
oauth_error_rate = Odpověděli jste na příliš mnoho požadavků. Několik minut počkejte a zkuste to znovu.
oauth_error_unreachable = Spojení s tuita se nepodařilo. Zkontrolujte připojení a zkuste to znovu.
oauth_home = Přejít na tuita
share_send_chat = Odeslat v chatu
share_download = Stáhnout média
share_download_n = Stáhnout média { $number }
share_chat_title = Odeslat příspěvek v chatu
share_chat_sent = Odesláno uživateli { $handle }
hover_card_label = Karta profilu: { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Typ médií
profile_media_all = Všechna média
profile_media_photos = Fotky
profile_media_videos = Videa
profile_empty_videos_title = Zatím žádná videa
profile_empty_videos_body = Zde se objeví příspěvky s videi.
account_add_existing = Přidat existující účet
rail_label = Vyhledávání a návrhy
rail_search_submit = Hledat
rail_show_more = Zobrazit více
rail_footer = Odkazy
chat_new = Nový chat
chat_search = Hledat v konverzacích
chat_search_none = Hledání neodpovídá žádná konverzace.
chat_start_title = Zahájit konverzaci
chat_start_body = Vyberte konverzaci ze seznamu nebo začněte novou.
explore_sections = Sekce Prozkoumat
explore_see_more = Zobrazit více
history_title = Historie
history_tabs = Sekce historie
history_likes_label = Oblíbené příspěvky
video_play = Přehrát
video_pause = Pozastavit
video_mute = Ztlumit
video_unmute = Zapnout zvuk
video_volume = Hlasitost
video_seek = Přetáčení
video_position = { $current } z { $total }
video_speed = Rychlost přehrávání
video_fullscreen = Celá obrazovka
video_exit_fullscreen = Ukončit celou obrazovku
video_pip = Obraz v obraze
video_exit_pip = Ukončit obraz v obraze
video_error = Toto video nelze přehrát.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Skrýt panel
lightbox_show_panel = Zobrazit panel
lightbox_view_post = Zobrazit příspěvek
lightbox_panel = Příspěvek a odpovědi
emotepick_tabs = Sady emoji
emotepick_tab_emoji = Standardní
emotepick_tab_custom = Vlastní
admin_emote_tab = Emotikony
admin_emote_title = Vlastní emotikony
admin_emote_desc = Emotikony patří celé instanci. Každý je může používat jako :shortcode: v příspěvcích, jménech a zprávách. Kód emotikony nelze přejmenovat; smažte emotikonu a nahrajte ji znovu.
admin_emote_shortcode = Kód
admin_emote_shortcode_hint = 2 až 32 písmen, číslic nebo podtržítek. Zobrazuje se jako :shortcode:.
admin_emote_invalid = Použijte jen písmena, číslice a podtržítka, 2 až 32 znaků.
admin_emote_category = Kategorie
admin_emote_category_hint = Volitelné. Seskupuje emotikony ve výběru.
admin_emote_file = Obrázek emotikony
admin_emote_file_hint = PNG, JPEG, WebP nebo GIF, do 256 KB. Přetáhněte sem soubor nebo ho vyberte.
admin_emote_file_type = Tento soubor není obrázek PNG, JPEG, WebP ani GIF.
admin_emote_file_size = Obrázek je příliš velký (max. 256 KB).
admin_emote_choose_file = Vybrat obrázek
admin_emote_change_file = Změnit obrázek
admin_emote_upload = Nahrát emotikonu
admin_emote_upload_failed = Emotikonu se nepodařilo nahrát. Zkuste to znovu.
admin_emote_uploaded = Přidáno :{ $shortcode }:.
admin_emote_list = Emotikony v této instanci
admin_emote_none = Zatím žádné vlastní emotikony.
admin_emote_no_category = Bez kategorie
admin_emote_category_for = Kategorie :{ $shortcode }:
admin_emote_saved = Kategorie uložena.
admin_emote_delete = Smazat :{ $shortcode }:
admin_emote_delete_title = Smazat tuto emotikonu?
admin_emote_delete_body = :{ $shortcode }: se přestane všude zobrazovat jako obrázek a zobrazí se jako prostý text. Příspěvky, které ji použily, si text ponechají.
admin_emote_delete_confirm = Smazat emotikonu
admin_emote_deleted = Emotikona smazána.
admin_act_log_emoji_create = Emotikona přidána
admin_act_log_emoji_update = Emotikona změněna
admin_act_log_emoji_delete = Emotikona smazána
e2ee_settings = Šifrované zprávy
e2ee_settings_desc = Heslová fráze, toto zařízení a reset
e2ee_status_label = Stav
e2ee_status_none = Vypnuto. Zapněte šifrování, abyste mohli odesílat a číst zprávy šifrované end-to-end.
e2ee_status_locked = Na tomto zařízení zamčeno. Zadejte heslovou frázi, abyste si tu mohli přečíst své zprávy.
e2ee_status_ready = Zapnuto a na tomto zařízení odemčeno.
e2ee_keys_error = Šifrovací klíče se nepodařilo zkontrolovat.
e2ee_setup_title = Zapnout šifrované zprávy
e2ee_setup_body = Zprávy lidem na tomto serveru se zašifrují ve vašem prohlížeči. Server ukládá jen text, který nemůže přečíst. Heslová fráze vám umožní odemknout zprávy na jiném zařízení.
e2ee_setup_warning = Heslovou frázi nelze obnovit. Když ji zapomenete, můžete šifrování resetovat, ale dřívější zprávy se ztratí.
e2ee_setup_submit = Zapnout
e2ee_setup_done = Šifrované zprávy jsou zapnuté
e2ee_unlock_title = Odemknout šifrované zprávy
e2ee_unlock_body = Zadejte heslovou frázi, abyste na tomto zařízení mohli číst a odesílat šifrované zprávy.
e2ee_unlock_submit = Odemknout
e2ee_unlock_wrong = Heslová fráze není správná.
e2ee_unlock_done = Odemčeno
e2ee_passphrase = Heslová fráze
e2ee_passphrase_confirm = Potvrďte heslovou frázi
e2ee_passphrase_hint = Alespoň 10 znaků.
e2ee_passphrase_short = Použijte alespoň 10 znaků.
e2ee_passphrase_mismatch = Heslové fráze se neshodují.
e2ee_failed = Něco se pokazilo. Zkuste to znovu.
e2ee_unsupported = Tento prohlížeč šifrované zprávy nepodporuje.
e2ee_forget = Zapomenout na tomto zařízení
e2ee_forget_hint = Odebere z tohoto prohlížeče váš klíč a kontakty, kterým důvěřujete. Heslová fráze je odemkne znovu.
e2ee_forget_done = Na tomto zařízení zapomenuto
e2ee_reset = Resetovat šifrování
e2ee_reset_hint = Vytvoří nový klíč. Zprávy, které jste dříve odeslali a přijali, přestanou být čitelné.
e2ee_reset_title = Resetovat šifrování?
e2ee_reset_body = Dřívější zprávy přestanou být čitelné, pro vás i pro lidi, kterým jste psali, a nelze je vrátit. Vaše kontakty budou požádány, aby novému klíči důvěřovaly. Zvolte novou heslovou frázi.
e2ee_reset_submit = Resetovat
e2ee_reset_done = Šifrování bylo resetováno
e2ee_note_sealed = Šifrováno end-to-end
e2ee_note_remote = Není šifrováno end-to-end: tento člověk je na jiném serveru.
e2ee_note_plain = Není šifrováno end-to-end.
e2ee_plain_mark = Nešifrováno
e2ee_preview = Šifrovaná zpráva
e2ee_line_sealed = Šifrovaná zpráva
e2ee_line_own_key = Odesláno na klíč, který už nemáte
e2ee_line_peer_key = Odesláno klíčem, kterému zatím nedůvěřujete
e2ee_line_unreadable = Nepodařilo se otevřít
e2ee_line_unsupported = Tento prohlížeč šifrované zprávy neotevře
e2ee_block_peer_off = Uživatel { $handle } zatím nezapnul šifrované zprávy.
e2ee_block_self_off = Nejdřív zapněte šifrované zprávy v Nastavení.
e2ee_block_unlock = Nejdřív odemkněte šifrované zprávy v Nastavení.
e2ee_block_downgrade = Uživatel { $handle } už dříve používal šifrované zprávy. Tato konverzace se nevrátí k nešifrovaným zprávám.
e2ee_block_changed = Abyste mohli psát uživateli { $handle }, důvěřujte jeho novému klíči.
e2ee_block_error = Šifrovací klíče se nepodařilo zkontrolovat.
e2ee_changed_title = { $name } má nový šifrovací klíč
e2ee_changed_body = Mohlo dojít k resetu šifrování nebo přechodu na nové zařízení, anebo se mezi vás někdo vkládá. Než novému klíči uvěříte, porovnejte nové bezpečnostní číslo s dotyčným. Dokud to neuděláte, odesílat nemůžete.
e2ee_changed_old = Bezpečnostní číslo, kterému jste důvěřovali
e2ee_changed_new = Nové bezpečnostní číslo
e2ee_trust = Důvěřovat novému klíči
e2ee_safety_open = Bezpečnostní číslo
e2ee_safety_title = Bezpečnostní číslo
e2ee_safety_body = Porovnejte toto číslo s uživatelem { $name } osobně nebo při hovoru, ne v této konverzaci. Pokud se shoduje na obou obrazovkách, nikdo se mezi vás nevkládá.

## 0031
composer_upload_failed = Nahrání se nezdařilo. Zkus to znovu.
composer_send_failed = Spojení s tuita se nezdařilo. Zkontroluj připojení a zkus to znovu.
composer_open_failed = Editor příspěvku se nepodařilo otevřít. Zkus to znovu.
