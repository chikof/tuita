app_name = tuita
loading = Завантаження
close = Закрити
dismiss = Закрити
notifications = Сповіщення
avatar_more = ще { $count }
char_counter_left =
    { $count ->
        [one] Залишився { $count } символ
        [few] Залишилося { $count } символи
        [many] Залишилося { $count } символів
       *[other] Залишилося { $count } символу
    }
char_counter_over =
    { $count ->
        [one] { $count } зайвий символ
        [few] { $count } зайві символи
        [many] { $count } зайвих символів
       *[other] { $count } зайвого символу
    }
not_found_title = Тут нічого немає
not_found_body = Цієї сторінки не існує або її переміщено.
not_found_back = Назад
tagline = Кожен допис у двох чорнилах.
error_title = Щось пішло не так
cancel = Скасувати
retry = Спробувати ще раз
back = Назад
view = Переглянути
follow = Стежити
following = Відстежується
follows_you = Стежить за вами
verified = Підтверджений обліковий запис
bot_account = Автоматизований обліковий запис
bot_label = Бот
link_copied = Посилання скопійовано в буфер обміну
text_copied = Текст скопійовано
time_now = щойно
time_minutes = { $n } хв
time_hours = { $n } год
time_days = { $n } д
nav_main = Головне
nav_home = Головна
nav_explore = Огляд
nav_notifications = Сповіщення
nav_bookmarks = Закладки
nav_profile = Профіль
nav_settings = Налаштування
nav_compose = Опублікувати
nav_menu = Меню
badge_unread = Непрочитаних: { $count }
account_menu = Облікові записи
account_switch = Змінити обліковий запис
account_switch_to = Перейти на { $handle }
account_add = Додати обліковий запис
account_logout = Вийти з { $handle }
account_logout_all = Вийти з усіх облікових записів
home_feed = Стрічка
home_for_you = Для вас
home_following = Відстежується
home_empty_title = Тут поки нічого немає
home_empty_body = Опублікуйте щось першими.
home_following_empty_title = У вашому колі тихо
home_following_empty_body = Стежте за людьми, щоб бачити в цій стрічці їхні дописи й поширення.
home_find_people = Знайти людей
new_posts =
    { $count ->
        [one] Показати { $count } новий допис
        [few] Показати { $count } нові дописи
        [many] Показати { $count } нових дописів
       *[other] Показати { $count } нового допису
    }
new_posts_many = Показати 99+ нових дописів
composer_title = Новий допис
composer_reply_title = Відповісти
composer_quote_title = Цитувати допис
composer_placeholder = Що відбувається?
composer_reply_placeholder = Опублікуйте відповідь
composer_text_label = Текст допису
composer_replying_to = Відповідь для { $handle }
composer_cw_label = Попередження про вміст
composer_cw_placeholder = Попередження про вміст, наприклад про спойлери
composer_cw_toggle = Додати попередження про вміст
composer_sensitive_toggle = Позначити медіа як чутливе
composer_options = Параметри допису
composer_policy_label = Хто може відповідати
composer_suggestions = Підказки
composer_tag_posts =
    { $count ->
        [one] { $compact } допис
        [few] { $compact } дописи
        [many] { $compact } дописів
       *[other] { $compact } допису
    }
composer_post = Опублікувати
composer_reply = Відповісти
policy_everyone = Відповідати можуть усі
policy_following = Відповідати можуть люди, за якими ви стежите
policy_mentioned = Відповідати можуть лише згадані вами люди
post_sent = Допис опубліковано
post_by = Допис від { $name }
post_reposted_by = { $name } поширює
post_pinned = Закріплений допис
post_replying_to = Відповідь для { $handle }
post_replying_unavailable = Відповідь на недоступний допис
post_unavailable = Цей допис недоступний.
post_menu = Більше
post_copy_link = Копіювати посилання
post_copy_text = Копіювати текст
post_pin = Закріпити в профілі
post_unpin = Відкріпити від профілю
post_pinned_toast = Закріплено у вашому профілі
post_unpinned = Відкріплено від вашого профілю
post_delete = Видалити допис
post_delete_title = Видалити допис?
post_delete_body = Цю дію не можна скасувати. Допис буде вилучено з вашого профілю, зі стрічок людей, які за вами стежать, і з пошуку.
post_deleted = Допис видалено
post_view_profile = Переглянути { $handle }
post_mute = Приглушити { $handle }
post_block = Заблокувати { $handle }
post_reply_label = Відповісти ({ $count })
post_reply_following_only = Відповідати можуть лише люди, за якими стежить автор.
post_reply_mentioned_only = Відповідати можуть лише люди, згадані в цьому дописі.
post_repost_menu = Параметри поширення
post_repost = Поширити
post_undo_repost = Скасувати поширення
post_quote = Цитувати
post_reposts_label = Поширення й цитування ({ $count })
post_like_label = Вподобати ({ $count })
post_bookmark_label = Додати в закладки
post_share = Поділитися або скопіювати посилання
cw_show = Показати
cw_hide = Приховати
cw_label = Попередження про вміст: { $reason }
muted_toast = { $handle } приглушено
unmuted_toast = Приглушення { $handle } скасовано
blocked_toast = { $handle } заблоковано
unblocked_toast = { $handle } розблоковано
block_title = Заблокувати { $handle }?
block_body = Ця людина не зможе стежити за вами, а ви не бачитимете її дописів. Вона про це не дізнається.
block_confirm = Заблокувати
notify_like = @{ $handle } вподобав(-ла) ваш допис
notify_repost = @{ $handle } поширив(-ла) ваш допис
notify_reply = @{ $handle } відповів(-ла) вам
notify_mention = @{ $handle } згадав(-ла) вас
notify_quote = @{ $handle } процитував(-ла) ваш допис
notify_follow = @{ $handle } стежить за вами
notify_follow_request = @{ $handle } просить дозволу стежити за вами
notify_poll = Опитування, у якому ви голосували, завершилося
notify_affiliation = @{ $handle } запрошує вас до афіліації
shortcuts_title = Комбінації клавіш
shortcut_group_navigation = Навігація
shortcut_group_actions = Дії
shortcut_or = /
shortcut_then = потім
shortcut_plus = +
shortcut_next_prev = Наступний / попередній допис
shortcut_open = Відкрити допис
shortcut_search = Пошук
shortcut_go_home = Головна
shortcut_go_explore = Огляд
shortcut_go_notifications = Сповіщення
shortcut_go_bookmarks = Закладки
shortcut_go_profile = Ваш профіль
shortcut_go_settings = Налаштування
shortcut_new_post = Новий допис
shortcut_like = Вподобати
shortcut_reply = Відповісти
shortcut_repost = Поширити або процитувати
shortcut_bookmark = Додати в закладки
shortcut_send = Опублікувати допис
shortcut_help = Показати цю довідку
auth_login_title = Увійти
auth_add_title = Додати ще один обліковий запис
auth_add_sub = Увійдіть, щоб перемикатися між кількома (до { $max }) обліковими записами на цьому пристрої без виходу.
auth_signup_title = Створити обліковий запис
auth_handle = Ім’я користувача
auth_password = Пароль
auth_display_name = Відображуване ім’я
auth_handle_hint = 3–15 літер, цифр або підкреслень
auth_password_hint = Щонайменше 8 символів
auth_consent_pre = Мені виповнилося щонайменше { $minAge } років, і я погоджуюся з{" "}
auth_consent_terms = Умовами використання
auth_consent_and = {" "}та{" "}
auth_consent_privacy = Політикою конфіденційності
auth_consent_end = {"."}
auth_consent_required = Прийміть Умови використання та Політику конфіденційності.
auth_signin = Увійти
auth_signin_add = Увійти й перемкнутися
auth_create = Створити обліковий запис
auth_to_signup = Вперше тут? Створіть обліковий запис
auth_to_login = Маєте обліковий запис? Увійдіть
auth_closed = Реєстрацію на цьому інстансі зараз закрито.
landing_nav = Підвал сайту
landing_rules = Правила спільноти
landing_stats = Просто зараз
landing_people = Люди
landing_posts = Дописи
landing_clock = Ваш місцевий час
landing_trends = У тренді
landing_trend =
    { $count ->
        [one] { $count } допис
        [few] { $count } дописи
        [many] { $count } дописів
       *[other] { $count } допису
    }
landing_join = Приєднуйтесь до { $name }
landing_join_sub = Створіть обліковий запис і публікуйте в двох чорнилах.
landing_closed = Реєстрацію зараз закрито. Уже маєте обліковий запис? Увійдіть.
landing_terms = Умови
landing_privacy = Конфіденційність
landing_source = Вихідний код
landing_powered = Працює на tuita { $version }
docs_terms_title = Умови використання
docs_privacy_title = Політика конфіденційності
docs_updated = Останнє оновлення: { $date }
docs_date = 5 листопада 2026 р.
docs_see_privacy = Див. також Політику конфіденційності.
docs_see_terms = Див. також Умови використання.
docs_contact_fallback = адміністратори інстансу (через особисте повідомлення або скаргу)
docs_terms_agreement_h = 1. Згода
docs_terms_agreement_p = Ці Умови використання («Умови») регулюють користування { $name }, інстансом tuita за адресою { $domain } («сервіс», «ми»). Створюючи обліковий запис або користуючись сервісом, ви погоджуєтеся з цими Умовами та нашою Політикою конфіденційності. Якщо ви не згодні, не користуйтеся сервісом.
docs_terms_eligibility_h = 2. Хто може користуватися
docs_terms_eligibility_p = Вам має бути щонайменше { $minAge } років або виповнитися мінімальний вік цифрової згоди за місцем вашого проживання, якщо він вищий. Ви маєте бути здатні укладати обов’язкові угоди, і чинне законодавство не повинно забороняти вам користуватися сервісом.
docs_terms_account_h = 3. Ваш обліковий запис
docs_terms_account_1 = Тримайте пароль у таємниці. Ви відповідаєте за дії у своєму обліковому записі.
docs_terms_account_2 = Ваше ім’я користувача не повинно видавати вас за іншу особу чи організацію або вводити людей в оману щодо того, хто ви.
docs_terms_account_3 = Негайно повідомте нас, якщо підозрюєте, що обліковий запис зламано. Інші сеанси можна завершити у розділі «Налаштування → Обліковий запис».
docs_terms_content_h = 4. Ваш вміст
docs_terms_content_p1 = Ви залишаєтеся власником того, що публікуєте. Ви надаєте нам всесвітню, невиключну, безоплатну ліцензію зберігати, показувати, відтворювати й поширювати ваш вміст лише для роботи та вдосконалення сервісу. Це охоплює надсилання його федеративним інстансам tuita (див. розділ 6). Ліцензія діє, доки ви не видалите вміст або обліковий запис, окрім копій, які вже доставлено іншим інстансам або збережено іншими людьми.
docs_terms_content_p2 = Ви відповідаєте за те, що публікуєте. Публікуйте лише те, чим маєте право ділитися.
docs_terms_rules_h = 5. Правила спільноти
docs_terms_rules_lead = Користуючись { $name }, ви погоджуєтеся не порушувати закон і ці правила:
docs_terms_rules_p = Ви також погоджуєтеся не атакувати сервіс, зокрема не обходити засоби безпеки, не збирати з нього дані у великих обсягах, не розсилати шкідливе ПЗ і не перевантажувати його.
docs_terms_federation_h = 6. Федерація
docs_terms_federation_p = { $name } може з’єднуватися («федеруватися») з іншими незалежними інстансами tuita, схваленими адміністраторами. Коли ви взаємодієте з людьми на іншому інстансі, туди надсилаються ваш публічний профіль, публічні дописи та сама взаємодія (наприклад, підписка, вподобання, поширення, відповідь чи особисте повідомлення). Інші інстанси ведуть інші люди за власними умовами. Ми не можемо контролювати чи видаляти вміст, який уже потрапив на інший інстанс, проте надсилаємо їм запити на видалення, коли ви щось видаляєте.
docs_terms_moderation_h = 7. Модерація та примус до дотримання
docs_terms_moderation_p = Модератори можуть вилучати вміст, обмежувати функції або призупиняти облікові записи, що порушують ці Умови чи закон. За можливості ми пояснимо причину. Будь-хто може поскаржитися на допис або обліковий запис. Скарги надходять модераторам цього інстансу. Ми можемо припинити федерацію з будь-яким інстансом у будь-який час.
docs_terms_termination_h = 8. Вихід
docs_terms_termination_p = Ви можете експортувати свої дані або видалити обліковий запис будь-коли у розділі «Налаштування → Обліковий запис». Ми можемо призупинити або припинити ваш доступ, якщо ви серйозно чи неодноразово порушуєте ці Умови. Ми також можемо закрити сервіс і постараємося завчасно попередити.
docs_terms_disclaimers_h = 9. Відмова від гарантій і відповідальність
docs_terms_disclaimers_p = Сервіс надається «як є» та «як доступний», без жодних гарантій, наскільки це дозволяє закон. Ми не відповідаємо за непрямі, випадкові чи наслідкові збитки, а також за вміст, опублікований користувачами чи іншими інстансами. Ніщо в цих Умовах не обмежує відповідальності, яку не можна обмежити законом, або ваших законних прав як споживача.
docs_terms_changes_h = 10. Зміни
docs_terms_changes_p = Ми можемо оновлювати ці Умови. Про суттєві зміни повідомимо в застосунку до їх набуття чинності. Якщо ви й далі користуєтеся сервісом, це означає, що ви приймаєте нові Умови.
docs_terms_contact_h = 11. Контакти
docs_terms_contact_lead = Питання щодо цих Умов:
docs_privacy_summary_h = Коротко
docs_privacy_summary_1 = Жодної реклами, жодних скриптів стеження, жодного продажу даних.
docs_privacy_summary_2 = Один файл cookie, щоб ви залишалися в системі. Налаштування теми зберігаються у вашому браузері.
docs_privacy_summary_3 = Публічні дописи бачать усі, і вони можуть потрапити на федеративні інстанси tuita.
docs_privacy_summary_4 = Особисті повідомлення між обліковими записами на цьому сервері мають наскрізне шифрування, щойно обоє його ввімкнуть: вони зберігаються як текст, якого сервер не може прочитати, тож витік бази даних чи погляд на збережені дані нічого не розкриє. Повідомлення людям на інших серверах і повідомлення, надіслані до ввімкнення шифрування, не шифруються таким чином, і адміністратори технічно можуть їх прочитати. Цей сервер також доставляє застосунок, який шифрує, тож адміністратор, який змінив би цей застосунок, міг би перехопити те, що ви вводите.
docs_privacy_summary_5 = Ви можете експортувати або видалити свої дані будь-коли.
docs_privacy_controller_h = 1. Хто ми
docs_privacy_controller_p = Ця політика стосується { $name } за адресою { $domain }, який ведуть адміністратори інстансу. Вони вирішують, як обробляються ваші дані.
docs_privacy_collect_h = 2. Що ми збираємо
docs_privacy_collect_1 = Дані облікового запису: ім’я користувача, відображуване ім’я, пароль (зберігається лише як хеш argon2id із сіллю) і дані профілю: опис, місце, вебсайт, аватар, банер, форма й колір.
docs_privacy_collect_2 = Вміст: дописи, зображення, відповіді, вподобання, поширення, закладки, підписки, блокування, приглушення та скарги.
docs_privacy_collect_3 = Особисті повідомлення: зберігаються на цьому сервері, щоб їх можна було доставити вам і другій людині. Повідомлення з наскрізним шифруванням зберігається як шифротекст, який ми не можемо прочитати; хто його надіслав, кому й коли, залишається нам видимим. Інші повідомлення зберігаються як звичайний текст. Ваш ключ шифрування зберігається тут лише у вигляді, заблокованому вашою парольною фразою, якої ми ніколи не бачимо.
docs_privacy_collect_4 = Дані сеансів: для кожного пристрою, з якого виконано вхід, user-agent браузера, IP-адреса та час останнього використання, щоб ви могли переглядати сеанси й завершувати їх.
docs_privacy_collect_5 = Короткочасні дані безпеки: IP-адреси, які кілька хвилин зберігаються в пам’яті для обмеження частоти запитів.
docs_privacy_collect_p = Ми не збираємо вашу електронну пошту, номер телефону, контакти чи місцезнаходження, якщо ви самі не вкажете їх у профілі.
docs_privacy_use_h = 3. Як ми це використовуємо
docs_privacy_use_p = Лише для роботи сервісу: показувати ваш профіль і дописи, формувати стрічки та сповіщення, доставляти повідомлення, захищати облікові записи, запобігати зловживанням і забезпечувати дотримання Умов. Стрічка «Для вас» ранжує дописи за вподобаннями, поширеннями, відповідями та віком. Вона не складає ваш профіль.
docs_privacy_visibility_h = 4. Що бачать інші
docs_privacy_visibility_p = Ваш профіль, дописи, відповіді, поширення, вподобання, читачі та підписки публічні для всіх, хто увійшов на цей інстанс, і для федеративних інстансів. Коли хтось ділиться посиланням на допис чи профіль, застосунки на кшталт Discord, Slack або iMessage можуть отримати попередній перегляд із публічним дописом, його зображеннями та публічними даними вашого профілю. Дописи за попередженням про вміст або позначені як чутливі показуються в попередньому перегляді без тексту й зображень. Закладки, блокування, приглушення, скарги та налаштування приватні. Якщо вимкнути «Показувати мене в підказках», люди все одно зможуть знайти вас за точним іменем користувача.
docs_privacy_federation_h = 5. Федерація
docs_privacy_federation_p = Коли ви стежите за кимось на іншому інстансі tuita, відповідаєте, згадуєте, вподобуєте, поширюєте чи пишете йому, ми надсилаємо туди потрібні дані: ваш публічний профіль, допис або взаємодію та текст особистих повідомлень. Особисті повідомлення на інші інстанси не мають наскрізного шифрування. Інстанси ваших читачів отримують ваші публічні дописи й оновлення профілю. Ці інстанси працюють незалежно й зберігають дані за власними правилами. Коли ви видаляєте допис або обліковий запис, ми просимо федеративні інстанси видалити свої копії, але гарантувати цього не можемо.
docs_privacy_cookies_h = 6. Cookie та локальне сховище
docs_privacy_cookies_p = Ми використовуємо один суворо необхідний сеансовий файл cookie HttpOnly. Ваша тема, колір, розмір тексту та параметри руху зберігаються в локальному сховищі браузера й нам не надсилаються.
docs_privacy_retention_h = 7. Як довго ми зберігаємо дані
docs_privacy_retention_1 = Вміст і дані облікового запису зберігаються, доки ви їх не видалите.
docs_privacy_retention_2 = Видалені дописи одразу зникають із публічного перегляду, а їхній текст і зображення стираються.
docs_privacy_retention_3 = Коли ви видаляєте обліковий запис, ми вилучаємо ваші дописи, вподобання, підписки, повідомлення та сеанси й знеособлюємо запис облікового запису.
docs_privacy_retention_4 = Сеанси спливають через 30 днів або одразу, коли ви виходите.
docs_privacy_retention_5 = Скарги до модераторів і журнал дій адміністраторів зберігаються стільки, скільки потрібно для безпеки сервісу.
docs_privacy_rights_h = 8. Ваші права
docs_privacy_rights_p = Залежно від місця проживання (наприклад, згідно з GDPR або CCPA) ви можете мати право на доступ до своїх даних, їх виправлення, експорт чи видалення, а також заперечувати проти обробки або обмежувати її. Більшість цього можна зробити самостійно у розділі «Налаштування → Обліковий запис»: змінити профіль, завантажити дані та видалити обліковий запис. З усіх інших питань звертайтеся:
docs_privacy_rights_end = Ви також можете поскаржитися місцевому органу із захисту даних.
docs_privacy_security_h = 9. Безпека
docs_privacy_security_p = Паролі хешуються за допомогою scrypt. Токени сеансів зберігаються лише як хеші. Запити між федеративними інстансами підписуються ключами Ed25519. Жодна система не є абсолютно безпечною, тому використовуйте унікальний пароль.
docs_privacy_children_h = 10. Діти
docs_privacy_children_p = Сервіс не призначений для дітей молодше мінімального віку, зазначеного в Умовах. Якщо ви вважаєте, що дитина має обліковий запис, поскаржтеся.
docs_privacy_changes_h = 11. Зміни
docs_privacy_changes_p = Зміни до цієї політики ми публікуватимемо тут і оновлюватимемо дату вище. Про суттєві зміни повідомимо в застосунку.
docs_privacy_contact_h = 12. Контакти
docs_privacy_contact_lead = Питання чи запити щодо конфіденційності:
profile_posts =
    { $count ->
        [one] { $compact } допис
        [few] { $compact } дописи
        [many] { $compact } дописів
       *[other] { $compact } допису
    }
profile_edit = Редагувати профіль
profile_more = Більше
profile_copy_link = Копіювати посилання на профіль
profile_mute = Приглушити { $handle }
profile_unmute = Скасувати приглушення { $handle }
profile_block = Заблокувати { $handle }
profile_unblock = Розблокувати { $handle }
profile_unblock_button = Розблокувати
profile_suspended = Цей обліковий запис призупинено. Його бачить лише команда.
profile_muted = Приглушено
profile_org = Організація
profile_role_admin = Адміністратор
profile_role_moderator = Модератор
profile_joined = Приєднався(-лася) { $date }
profile_following_label = Відстежується
profile_followers =
    { $count ->
        [one] Читач
        [few] Читачі
        [many] Читачів
       *[other] Читача
    }
profile_counts = Кількість підписок
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Читає { $a }
            }
        [2]
            { $others ->
               *[other] Читають { $a } і { $b }
            }
       *[other]
            { $others ->
                [one] Читають { $a }, { $b } і ще { $others } із тих, за ким ви стежите
                [few] Читають { $a }, { $b } і ще { $others } із тих, за ким ви стежите
                [many] Читають { $a }, { $b } і ще { $others } із тих, за ким ви стежите
               *[other] Читають { $a }, { $b } і ще { $others } із тих, за ким ви стежите
            }
    }
profile_banner = Банер профілю
profile_tabs = Розділи профілю
profile_tab_posts = Дописи
profile_tab_replies = Відповіді
profile_tab_media = Медіа
profile_tab_likes = Вподобання
profile_posts_label = Дописи
profile_empty_posts_self_title = Ви ще нічого не публікували
profile_empty_posts_self_body = Тут з’являться ваші дописи й поширення.
profile_empty_posts_title = { $handle } ще нічого не публікував(-ла)
profile_empty_posts_body = Коли з’являться дописи, їх буде видно тут.
profile_empty_replies_title = Відповідей поки немає
profile_empty_replies_body = Тут з’являться відповіді.
profile_empty_media_title = Фото поки немає
profile_empty_media_body = Тут з’являться дописи із зображеннями.
profile_empty_likes_title = Вподобань поки немає
profile_empty_likes_body = Тут з’являться вподобані дописи.
profile_blocked_title = Ви заблокували { $handle }
profile_blocked_body = Ви не бачитимете дописів цієї людини, а вона не зможе стежити за вами чи писати вам.
profile_hidden_title = Ви не можете бачити ці дописи
profile_hidden_body = { $handle } обмежив(-ла), хто може бачити профіль.
follows_tabs = Списки підписок
follows_tab_followers = Читачі
follows_tab_following = Відстежується
follows_label_followers = Читачі
follows_label_following = Відстежується
follows_empty_followers_title = Читачів поки немає
follows_empty_followers_body = Коли хтось почне стежити за цим обліковим записом, тут з’являться читачі.
follows_empty_following_title = Ні за ким не стежить
follows_empty_following_body = Тут з’являться ті, за ким стежить цей обліковий запис.
thread_title = Допис
thread_doc_title = { $name }: «{ $text }»
thread_views =
    { $count ->
        [one] Перегляд
        [few] Перегляди
        [many] Переглядів
       *[other] Перегляду
    }
thread_reposts =
    { $count ->
        [one] Поширення
        [few] Поширення
        [many] Поширень
       *[other] Поширення
    }
thread_quotes =
    { $count ->
        [one] Цитування
        [few] Цитування
        [many] Цитувань
       *[other] Цитування
    }
thread_likes =
    { $count ->
        [one] Вподобання
        [few] Вподобання
        [many] Вподобань
       *[other] Вподобання
    }
thread_stats = Статистика допису
thread_replies = Відповіді
thread_hidden = Приховані відповіді ({ $count })
thread_hidden_note = Автор приховав ці відповіді.
thread_list_likes = Вподобали
thread_list_reposts = Поширили
thread_list_quotes = Цитування
thread_list_empty = Поки нікого
tag_empty_title = Немає дописів із { $tag }
tag_empty_body = Використайте його першими.
tag_posts_label = Дописи з тегом { $tag }
bookmarks_title = Закладки
bookmarks_label = Дописи в закладках
bookmarks_empty_title = Зберігайте дописи на потім
bookmarks_empty_body = Торкніться значка закладки на будь-якому дописі, щоб зберегти його тут. Закладки бачите лише ви.
settings_account = Ваш обліковий запис
settings_account_desc = Профіль і пароль
settings_profile = Редагувати профіль
settings_profile_desc = Ім’я, опис, форма аватара й чорнило
settings_privacy = Конфіденційність і безпека
settings_privacy_desc = Повідомлення, виявлення, приглушені слова, заблоковані та приглушені облікові записи
settings_notifications = Сповіщення
settings_notifications_desc = Виберіть, про що отримувати сповіщення
settings_appearance = Вигляд
settings_appearance_desc = Тема, чорнило, розмір тексту, контрастність і рух
settings_language = Мова
settings_language_desc = Мова інтерфейсу
settings_language_note = Зберігається в цьому браузері та у вашому обліковому записі, якщо ви увійшли.
settings_about = Про цей інстанс
settings_about_desc = Правила, умови, конфіденційність, версія й клавіатурні скорочення
settings_logout = Вийти
settings_account_profile = Профіль
settings_account_joined = Приєднався(-лася) { $date }
settings_account_edit = Редагувати профіль
settings_password_title = Змінити пароль
settings_password_current = Поточний пароль
settings_password_new = Новий пароль
settings_password_hint = Щонайменше 8 символів. Інші пристрої буде від’єднано.
settings_password_confirm = Підтвердьте новий пароль
settings_password_submit = Оновити пароль
settings_password_mismatch = Нові паролі не збігаються.
settings_password_updated = Пароль оновлено. Інші сеанси завершено.
settings_sessions_title = Пристрої та сеанси
settings_sessions_desc = Завершіть усі сеанси, яких не впізнаєте, а потім змініть пароль.
settings_device_on = { $browser } на { $os }
settings_device_unknown = Невідомий пристрій
settings_session_current = Цей пристрій
settings_session_last_active = Остання активність: { $time }
settings_session_sign_out = Вийти
settings_session_sign_out_device = Вийти на пристрої { $device }
settings_session_signed_out = Сеанс завершено
settings_sessions_revoke_all = Вийти на всіх інших пристроях
settings_sessions_revoke_title = Вийти на інших пристроях?
settings_sessions_revoke_body = З усіх пристроїв, крім цього, буде виконано вихід.
settings_sessions_revoke_confirm = Вийти на інших пристроях
settings_sessions_revoked =
    { $count ->
        [one] Завершено { $count } сеанс
        [few] Завершено { $count } сеанси
        [many] Завершено { $count } сеансів
       *[other] Завершено { $count } сеансу
    }
settings_profile_preview = Попередній перегляд
settings_profile_preview_alt = Попередній перегляд аватара
settings_profile_uploads = Завантаження аватара й банера з’явиться згодом. Поки що використовуються згенеровані аватар і банер.
settings_profile_details = Подробиці
settings_profile_name = Відображуване ім’я
settings_profile_bio = Опис
settings_profile_location = Місцезнаходження
settings_profile_website = Вебсайт
settings_profile_shape = Форма аватара
settings_profile_ink = Чорнило профілю
settings_profile_save = Зберегти профіль
settings_profile_saved = Профіль оновлено
settings_shape_square = Квадрат
settings_shape_circle = Коло
settings_shape_squircle = Сквіркл
settings_shape_octagon = Восьмикутник
settings_ink_bone = Кістяне
settings_ink_phosphor = Фосфорне
settings_ink_amber = Бурштинове
settings_ink_cyan = Блакитне
settings_ink_rose = Рожеве
settings_dm_title = Особисті повідомлення
settings_dm_desc = Хто може почати з вами розмову
settings_dm_label = Хто може вам писати
settings_dm_everyone = Усі
settings_dm_following = Люди, за якими я стежу
settings_dm_none = Ніхто
settings_dm_saved = Налаштування повідомлень збережено
settings_discover_title = Видимість
settings_discoverable = Показувати мене в підказках і пошуку
settings_discoverable_hint = Вас усе одно можна знайти за точним іменем користувача
settings_show_client = Показувати, з якого пристрою я публікую
settings_show_client_hint = Додає пристрій під вашими новими дописами
settings_words_title = Приглушені слова
settings_words_desc = Приховуйте зі стрічок, пошуку й відповідей дописи з цими словами, фразами чи #хештегами. Ваші власні дописи ніколи не приховуються.
settings_words_label = Слово для приглушення
settings_words_placeholder = Додайте слово, фразу чи #хештег
settings_words_add = Приглушити
settings_words_remove = Скасувати приглушення { $word }
settings_words_none = Приглушених слів немає.
settings_words_limit = Можна приглушити до { $count } слів.
settings_blocked_title = Заблоковані облікові записи
settings_blocked_desc = Вони не можуть стежити за вами, писати вам, бачити ваші дописи чи взаємодіяти з ними. Вони про це не дізнаються.
settings_blocked_empty = Ви нікого не заблокували.
settings_muted_title = Приглушені облікові записи
settings_muted_desc = Ви не бачитимете їхніх дописів і сповіщень. Вони й надалі можуть стежити за вами та писати вам.
settings_muted_empty = Ви нікого не приглушили.
settings_unblock = Розблокувати
settings_unmute = Скасувати приглушення
settings_unblock_user = Розблокувати { $handle }
settings_unmute_user = Скасувати приглушення { $handle }
settings_notif_title = Сповіщати мене про
settings_notif_desc = Приглушені та заблоковані облікові записи ніколи не надсилають вам сповіщень.
settings_notif_like = Вподобання
settings_notif_like_hint = Коли хтось вподобає ваш допис
settings_notif_repost = Поширення
settings_notif_repost_hint = Коли хтось поширить ваш допис
settings_notif_reply = Відповіді
settings_notif_reply_hint = Коли хтось вам відповість
settings_notif_mention = Згадки
settings_notif_mention_hint = Коли хтось вас згадає
settings_notif_quote = Цитування
settings_notif_quote_hint = Коли хтось процитує ваш допис
settings_notif_follow = Нові читачі
settings_notif_follow_hint = Коли хтось почне стежити за вами
settings_desktop_title = Системні сповіщення
settings_desktop_desc = Показується як системне сповіщення, лише коли ця вкладка у фоновому режимі.
settings_desktop_state = Дозвіл: { $state }
settings_desktop_allowed = Дозволено
settings_desktop_blocked = Заблоковано
settings_desktop_not_asked = Не запитувалося
settings_desktop_enable = Увімкнути
settings_desktop_blocked_toast = Сповіщення для цього сайту заблоковано в налаштуваннях браузера.
settings_desktop_unsupported = Цей браузер не підтримує сповіщень на робочому столі.
settings_theme_title = Тема
settings_appearance_note = Зберігається на цьому пристрої та застосовується одразу.
settings_theme_mode = Тема
settings_theme_system = Системна
settings_theme_light = Світла
settings_theme_dark = Темна
settings_theme_ink = Чорнило
settings_theme_contrast = Контрастність
settings_contrast_normal = Звичайна
settings_contrast_more = Підвищена
settings_reading_title = Текст і рух
settings_theme_text_size = Розмір тексту
settings_theme_motion = Рух
settings_motion_system = Як у системі
settings_motion_reduce = Зменшити
settings_preview_title = Попередній перегляд
settings_preview_text = Ось як виглядатимуть дописи з вашими налаштуваннями.
settings_about_domain = Домен
settings_about_version = Версія
settings_about_contact = Контакти
settings_about_terms = Умови використання
settings_about_privacy = Політика конфіденційності
settings_about_source = Вихідний код
settings_about_rules = Правила
explore_search_label = Пошук дописів, людей і тегів
explore_search_placeholder = Шукайте дописи, людей, #теги
explore_clear = Очистити пошук
explore_tips = Поради з пошуку
explore_tips_title = Оператори пошуку
explore_op_from = Дописи облікового запису
explore_op_to = Відповіді або згадки облікового запису
explore_op_media = Дописи із зображеннями (або has:poll, has:links)
explore_op_since = Опубліковано в зазначену дату або пізніше
explore_op_until = Опубліковано в зазначену дату або раніше
explore_op_phrase = Слова в такому порядку
explore_op_exclude = Виключити дописи із зазначеним словом
explore_results = Результати пошуку
explore_tab_posts = Дописи
explore_tab_people = Люди
explore_tab_media = Медіа
explore_people = Люди
explore_posts = Дописи
explore_recent = Нещодавні пошуки
explore_recent_clear = Очистити
explore_trending = У тренді
explore_trend_posts =
    { $count ->
        [one] { $compact } допис
        [few] { $compact } дописи
        [many] { $compact } дописів
       *[other] { $compact } допису
    }
explore_trends_empty = Трендів поки немає. Використайте #хештег!
explore_suggestions = Кого читати
explore_suggestions_empty = Ви стежите за всіма. Вражає.
explore_no_posts_title = Немає дописів для «{ $q }»
explore_no_posts_body = Спробуйте інше слово або хештег.
explore_no_people_title = Немає людей для «{ $q }»
explore_no_people_body = Спробуйте ім’я, ім’я користувача або @користувач@інстанс, щоб знайти людину з іншого інстансу.
explore_no_media_title = Немає зображень для «{ $q }»
explore_no_media_body = Тут з’являться відповідні дописи із зображеннями.
notif_filter = Фільтр сповіщень
notif_tab_all = Усі
notif_tab_mentions = Згадки
notif_empty_all_title = Сповіщень поки немає
notif_empty_all_body = Тут з’являтимуться вподобання, відповіді, поширення та нові читачі.
notif_empty_mentions_title = Згадок поки немає
notif_empty_mentions_body = Тут з’являються відповіді, згадки та цитування ваших дописів.
notif_who_two = { $first } і { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } і ще { $count }
        [few] { $first }, { $second } і ще { $count }
        [many] { $first }, { $second } і ще { $count }
       *[other] { $first }, { $second } і ще { $count }
    }
notif_verb_like =
    { $count ->
        [one] вподобав(-ла) ваш допис
        [few] вподобали ваш допис
        [many] вподобали ваш допис
       *[other] вподобали ваш допис
    }
notif_verb_repost =
    { $count ->
        [one] поширив(-ла) ваш допис
        [few] поширили ваш допис
        [many] поширили ваш допис
       *[other] поширили ваш допис
    }
notif_verb_follow =
    { $count ->
        [one] почав(-ла) стежити за вами
        [few] почали стежити за вами
        [many] почали стежити за вами
       *[other] почали стежити за вами
    }
notif_verb_follow_request =
    { $count ->
        [one] просить дозволу стежити за вами
        [few] просять дозволу стежити за вами
        [many] просять дозволу стежити за вами
       *[other] просять дозволу стежити за вами
    }
notif_verb_reply =
    { $count ->
        [one] відповів(-ла) вам
        [few] відповіли вам
        [many] відповіли вам
       *[other] відповіли вам
    }
notif_verb_mention =
    { $count ->
        [one] згадав(-ла) вас
        [few] згадали вас
        [many] згадали вас
       *[other] згадали вас
    }
notif_verb_quote =
    { $count ->
        [one] процитував(-ла) ваш допис
        [few] процитували ваш допис
        [many] процитували ваш допис
       *[other] процитували ваш допис
    }
notif_verb_poll =
    { $count ->
        [one] завершив(-ла) опитування, у якому ви голосували
        [few] завершили опитування, у якому ви голосували
        [many] завершили опитування, у якому ви голосували
       *[other] завершили опитування, у якому ви голосували
    }
notif_verb_affiliation =
    { $count ->
        [one] запрошує вас до афіліації
        [few] запрошують вас до афіліації
        [many] запрошують вас до афіліації
       *[other] запрошують вас до афіліації
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Запланувати
composer_post_all = Опублікувати все
composer_err_type = Цей файл не є зображенням чи відео, які підтримує tuita (PNG, JPEG, GIF, WebP, MP4, WebM або MOV).
composer_err_image_size = Зображення завелике (макс. 10 МБ).
composer_err_video_size = Відео може мати до { $mb } МБ.
composer_err_max_images = Можна додати до 4 зображень.
composer_err_exclusive = Допис може містити відео, зображення або опитування, але лише щось одне.
composer_extra_placeholder = Додати ще один допис
composer_extra_label = Допис ланцюжка { $n }
composer_extra_remove = Вилучити допис ланцюжка { $n }
composer_schedule_chip = Буде опубліковано { $time }
composer_schedule_clear = Не планувати
composer_add_images = Додати зображення
composer_add_video = Додати відео
composer_add_poll = Додати опитування
composer_add_post = Додати ще один допис
composer_scheduled = Заплановано на { $time }
composer_thread_sent = Ланцюжок із { $count } дописів опубліковано
composer_remove_video = Вилучити відео
composer_remove_image = Вилучити зображення
composer_uploading = Завантаження
composer_uploading_video = Завантаження відео… { $percent }%
composer_alt_video = Опишіть це відео
composer_alt_image = Опишіть це зображення
composer_alt_label = Опис (альтернативний текст) вкладення { $index }
composer_emoji = Додати емодзі
poll_length_hours = { $n } год
poll_length_days = { $n } д
poll_choice = Варіант { $n }
poll_choice_optional = Варіант { $n } (необов’язково)
poll_remove_choice = Вилучити варіант { $n }
poll_add_choice = Додати варіант
poll_length = Тривалість опитування
poll_multiple_label = Дозволити кілька варіантів
poll_remove = Вилучити опитування
schedule_title = Запланувати допис
schedule_hour = Через годину
schedule_evening = Сьогодні ввечері
schedule_tomorrow = Завтра зранку
schedule_week = Через тиждень
schedule_window = Виберіть час у межах від хвилини до 90 днів наперед.
schedule_custom = Вибрати дату й час
schedule_set = Встановити
schedule_clear = Не планувати
schedule_view = Переглянути заплановані дописи
dropzone_drop = Відпустіть, щоб додати
dropzone_reject = Цей файл не підходить
nav_messages = Повідомлення
nav_lists = Списки
nav_scheduled = Заплановано
notify_message = @{ $handle } надіслав(-ла) вам повідомлення
shortcut_go_messages = Повідомлення
shortcut_go_lists = Списки
cropper_stage = Область обрізання. Перетягніть, щоб перемістити зображення. Стрілками можна зсувати його, а плюсом і мінусом масштабувати.
cropper_hint = Перетягніть, щоб перемістити. Зведіть пальці, прокрутіть або скористайтеся повзунком, щоб змінити масштаб.
cropper_zoom = Масштаб
cropper_zoom_in = Збільшити
cropper_zoom_out = Зменшити
cropper_rotate = Повернути
cropper_reset = Скинути
cropper_apply = Застосувати
image_field_avatar = Зображення профілю
image_field_banner = Банер
image_field_edit_avatar = Редагувати фото профілю
image_field_edit_banner = Редагувати банер
image_field_change = Змінити
image_field_remove = Вилучити
image_field_hint = PNG, JPEG, GIF або WebP до 10 МБ. GIF стане нерухомим зображенням.
image_field_not_image = Цей файл не є зображенням, яке підтримує tuita (PNG, JPEG, GIF або WebP).
image_field_too_large = Зображення завелике (макс. 10 МБ).
image_field_uploading = Завантаження
image_field_saved = Зображення збережено.
image_field_removed = Зображення вилучено.
edit_title = Редагувати допис
edit_note = Допис можна редагувати протягом 30 хвилин після публікації. Люди бачать, що допис змінено, і його попередні версії.
edit_save = Зберегти
edit_saved = Допис оновлено
edit_edited = Змінено
edit_history_title = Історія змін
edit_history_current = Поточна версія
edit_history_earlier = Попередня версія
edit_history_original = Оригінал
report_post_title = Поскаржитися на допис
report_user_title = Поскаржитися на { $handle }
report_intro = Що не так? Модератори цього інстансу розглянуть вашу скаргу. { $handle } не побачить, хто поскаржився.
report_reason = Причина
report_note = Ще щось? (необов’язково)
report_send = Надіслати скаргу
report_sent = Дякуємо. Вашу скаргу надіслано модераторам.
report_duplicate = Ви вже скаржилися на це. Модератори переглянуть скаргу.
report_spam = Спам
report_spam_body = Небажана реклама, шахрайство, повторюваний або оманливий вміст
report_harassment = Цькування
report_harassment_body = Переслідування, цькування чи погрози комусь
report_hate = Ненависть
report_hate_body = Нападає на людей через те, ким вони є
report_violence = Насильство або самоушкодження
report_violence_body = Погрози, прославлення насильства, заохочення до самоушкодження
report_misinformation = Дезінформація
report_misinformation_body = Неправдиві твердження, що можуть зашкодити
report_impersonation = Видавання себе за іншого
report_impersonation_body = Видавання себе за когось іншого
report_other = Щось інше
report_other_body = Порушує правила в інший спосіб
post_edit = Редагувати допис
post_edit_history = Історія змін
post_report = Поскаржитися на допис
post_hide_reply = Приховати відповідь
post_unhide_reply = Показати відповідь
post_share_menu = Поділитися…
post_reply_hidden = Прихована відповідь
post_reply_hidden_toast = Відповідь приховано
post_reply_unhidden_toast = Відповідь знову видно
sensitive_label = Чутливі медіа
sensitive_show = Показати
sensitive_hide = Приховати чутливі медіа
media_label =
    { $count ->
        [one] { $count } вкладення
        [few] { $count } вкладення
        [many] { $count } вкладень
       *[other] { $count } вкладення
    }
media_video = Відео
media_open = Відкрити зображення { $index } з { $count }
lightbox_title = Перегляд медіа
lightbox_prev = Назад
lightbox_next = Далі
poll_vote = Проголосувати
poll_your_vote = Ваш голос
poll_votes =
    { $count ->
        [one] { $compact } голос
        [few] { $compact } голоси
        [many] { $compact } голосів
       *[other] { $compact } голосу
    }
poll_final = Остаточні результати
poll_multiple = кілька варіантів
poll_left_minutes = залишилося { $n } хв
poll_left_hours = залишилося { $n } год
poll_left_days = залишилося { $n } д
pull_refresh = Потягніть, щоб оновити
messages_title = Повідомлення
messages_new = Нове повідомлення
messages_label = Розмови
messages_empty_title = Почніть розмову
messages_empty_body = Повідомлення, які ви надсилаєте й отримуєте, з’являться тут.
messages_you_prefix = Ви: { $text }
messages_unread = Непрочитаних: { $count }
messages_new_search = Шукати людей
messages_new_results = Люди
messages_new_none = Нікого не знайдено.
chat_load_older = Завантажити попередні повідомлення
chat_intro = Це початок вашої розмови.
chat_label = Розмова з { $name }
chat_delete = Видалити
chat_seen = Переглянуто
chat_delete_title = Видалити це повідомлення?
chat_delete_body = Воно зникне для вас обох.
chat_doc_title = Розмова з { $name }
chat_error_title = Не вдалося відкрити розмову
chat_placeholder = Напишіть повідомлення
chat_input_label = Повідомлення
chat_send = Надіслати
lists_title = Списки
lists_new = Новий список
lists_edit = Редагувати список
lists_label = Ваші списки
lists_empty_title = Списків поки немає
lists_empty_body = Згрупуйте людей у списки, щоб читати їхні дописи окремо.
lists_members =
    { $count ->
        [one] { $compact } учасник
        [few] { $compact } учасники
        [many] { $compact } учасників
       *[other] { $compact } учасника
    }
lists_private = Приватний
lists_name = Назва
lists_description = Опис
lists_private_switch = Приватний список
lists_save = Зберегти
lists_picker_title = Додати { $handle } до списків
lists_picker_none = У вас поки немає списків.
lists_delete = Видалити список
lists_delete_title = Видалити цей список?
lists_delete_body = Список «{ $name }» буде вилучено. Це не вплине на людей у ньому.
lists_by = Список від { $handle }
lists_tab_posts = Дописи
lists_tab_members = Учасники
lists_tabs = Розділи списку
lists_error_title = Не вдалося відкрити список
lists_posts_label = Дописи списку
lists_posts_empty_title = Тут поки нічого немає
lists_posts_empty_body = Тут з’являтимуться дописи учасників цього списку.
lists_members_label = Учасники списку
lists_members_empty_title = Учасників немає
lists_members_empty_body = Додайте людей з їхніх профілів.
lists_remove = Вилучити
scheduled_title = Заплановані дописи
scheduled_label = Заплановані дописи
scheduled_status_pending = Заплановано
scheduled_status_published = Опубліковано
scheduled_status_failed = Не вдалося
scheduled_cw = Попередження про вміст: { $text }
scheduled_poll = Опитування з варіантами ({ $count }), триває { $hours } год
scheduled_change_time = Змінити час
scheduled_post_now = Опублікувати зараз
scheduled_remove = Вилучити
scheduled_window_error = Виберіть час у межах від хвилини до 90 днів наперед.
scheduled_when = Опублікувати
scheduled_zone = Часовий пояс: { $zone }
scheduled_save = Зберегти
scheduled_posted = Опубліковано.
scheduled_cancel_title = Скасувати цей запланований допис?
scheduled_cancel_body = Його не буде опубліковано.
scheduled_cancel_confirm = Скасувати допис
scheduled_empty_title = Нічого не заплановано
scheduled_empty_body = Дописи, які ви плануєте, чекають тут свого часу.
profile_message = Повідомлення
profile_add_to_lists = Додати до списків
media_alt = ALT
profile_report = Поскаржитися на { $handle }
chart_show_table = Показати «{ $title }» як таблицю
chart_col_day = День
chart_col_hour = Година (UTC)
chart_empty = { $title }: поки нема чого показати.
chart_bar_summary = { $title }: усього { $total }, найбільше — { $label } ({ $peak }).
chart_spark_summary = { $title }: усього { $total }, { $label } — { $last }.
chart_heat_summary = { $title }: усього { $total }, найактивніше — { $label } о { $hour } ({ $peak }).
chart_heat_less = Менше
chart_heat_more = Більше
announce_region = Оголошення
announce_dismiss = Закрити оголошення
announce_more = Докладніше
announce_tone_info = Інформація
announce_tone_success = Гарні новини
announce_tone_warning = Попередження
admin_stat_people = Люди
admin_stat_people_sub = { $count } активних цього тижня
admin_stat_posts_today = Дописи, 24 год
admin_stat_posts_total = усього { $count }
admin_stat_reports = Відкриті скарги
admin_stat_reports_open = Потребує розгляду
admin_stat_reports_clear = Усе гаразд
admin_stat_signups_today = Реєстрації сьогодні
admin_stat_signups_sub = { $count } за 14 днів
admin_stat_suspended = Призупинені
admin_stat_suspended_sub = облікових записів
admin_chart_posts = Дописи за день
admin_chart_signups = Реєстрації за день
admin_chart_reports = Скарги за день
admin_chart_heat = Активність за днями тижня й годинами
nav_admin = Адміністрування
admin_title = Адміністрування
admin_sections = Розділи адміністрування
admin_tab_overview = Огляд
admin_tab_reports = Скарги
admin_tab_users = Люди
admin_tab_instance = Інстанс
admin_tab_federation = Федерація
admin_tab_announcements = Оголошення
admin_tab_invites = Запрошення
admin_tab_filter = Фільтр вмісту
admin_tab_audit = Журнал дій
admin_denied_title = Лише для команди
admin_denied_body = Цей розділ призначено для адміністраторів і модераторів цього інстансу.
admin_denied_home = На головну
admin_unknown = невідомо
admin_tag_active = Активний
admin_tag_suspended = Призупинений
admin_tag_deleted = Видалений
admin_tag_org = Організація
admin_audit_system = система
admin_audit_empty = Поки нічого не записано
admin_report_status = Стан скарги
admin_report_open = Відкриті
admin_report_resolved = Розглянуті
admin_reports_empty_open = Відкритих скарг немає
admin_reports_empty_open_body = Коли люди скаржитимуться на дописи чи облікові записи, скарги з’являться тут.
admin_reports_empty_resolved = Розглянутих скарг поки немає
admin_report_by = скаржиться { $handle }
admin_report_same_target = Відкритих скарг: { $count }
admin_report_media =
    { $count ->
        [one] { $count } вкладення
        [few] { $count } вкладення
        [many] { $count } вкладень
       *[other] { $count } вкладення
    }
admin_report_poll = Містить опитування
admin_report_dismiss = Закрити
admin_report_delete_post = Видалити допис
admin_report_suspend = Призупинити обліковий запис
admin_report_delete_suspend = Видалити й призупинити
admin_report_confirm_title = Виконати дію?
admin_report_confirm_body = { $action }. Це буде записано в журнал дій.
admin_report_confirm = Підтвердити
admin_report_resolution = { $action }, виконав(-ла) { $handle } · { $time }
admin_res_dismiss = Закрито
admin_res_delete_post = Допис видалено
admin_res_suspend_user = Обліковий запис призупинено
admin_res_delete_and_suspend = Допис видалено, обліковий запис призупинено
admin_users_search = Шукати людей
admin_users_search_placeholder = Ім’я або ім’я користувача
admin_users_show = Показати
admin_users_empty = Людей не знайдено
admin_filter_local = Локальні
admin_filter_remote = Віддалені
admin_filter_staff = Команда
admin_filter_verified = Підтверджені
admin_filter_suspended = Призупинені
admin_filter_all = Усі
admin_user_details = Подробиці про { $handle }
admin_user_menu = Керувати { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } допис
        [few] { $compact } дописи
        [many] { $compact } дописів
       *[other] { $compact } допису
    }
admin_user_followers =
    { $count ->
        [one] { $compact } читач
        [few] { $compact } читачі
        [many] { $compact } читачів
       *[other] { $compact } читача
    }
admin_user_joined = приєднався(-лася) { $time }
admin_user_active = активність { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } відкрита скарга
        [few] { $compact } відкриті скарги
        [many] { $compact } відкритих скарг
       *[other] { $compact } відкритої скарги
    }
admin_act_verify = Підтвердити
admin_act_unverify = Зняти підтвердження
admin_act_org = Зробити організацією
admin_act_unorg = Зняти статус організації
admin_act_role_user = Зняти роль команди
admin_act_role_moderator = Призначити модератором
admin_act_role_admin = Призначити адміністратором
admin_act_suspend = Призупинити
admin_act_unsuspend = Відновити
admin_act_purge = Видалити всі дописи
admin_act_rename = Змінити ім’я користувача
admin_act_reset_name = Скинути відображуване ім’я
admin_act_clear_bio = Очистити опис
admin_act_delete = Видалити обліковий запис
admin_done_verified = { $handle } підтверджено
admin_done_unverified = Підтвердження знято
admin_done_org = { $handle } тепер організація
admin_done_unorg = Статус організації знято
admin_done_role = Роль оновлено
admin_done_suspended = { $handle } призупинено
admin_done_unsuspended = { $handle } відновлено
admin_done_purged =
    { $count ->
        [one] Видалено { $count } допис
        [few] Видалено { $count } дописи
        [many] Видалено { $count } дописів
       *[other] Видалено { $count } допису
    }
admin_done_name_reset = Відображуване ім’я скинуто
admin_done_bio_cleared = Опис очищено
admin_done_deleted = { $handle } видалено
admin_done_renamed = Тепер { $handle }
admin_role_phrase_user = звичайним користувачем
admin_role_phrase_moderator = модератором
admin_role_phrase_admin = адміністратором
admin_role_title = Змінити роль { $handle }?
admin_role_body = Ця людина стане { $role }.
admin_role_confirm = Змінити роль
admin_unorg_title = Зняти статус організації з { $handle }?
admin_unorg_body = Усі афіліації буде припинено.
admin_unorg_confirm = Зняти
admin_suspend_title = Призупинити { $handle }?
admin_suspend_body = Цю людину буде виведено з системи, а її вміст залишатиметься прихованим, доки ви не відновите обліковий запис.
admin_suspend_confirm = Призупинити
admin_purge_title = Видалити всі дописи від { $handle }?
admin_purge_body = Цю дію не можна скасувати.
admin_purge_confirm = Видалити все
admin_reset_name_title = Скинути відображуване ім’я { $handle }?
admin_reset_name_body = Його буде замінено іменем користувача.
admin_reset_name_confirm = Скинути
admin_delete_title = Видалити { $handle }?
admin_delete_body = Дописи, підписки й повідомлення буде вилучено, а обліковий запис знеособлено. Цю дію не можна скасувати.
admin_delete_confirm = Видалити обліковий запис
admin_rename_title = Змінити ім’я користувача { $handle }
admin_rename_note = Посилання на старе ім’я користувача перестануть працювати.
admin_rename_handle = Нове ім’я користувача
admin_rename_reset_name = Також скинути відображуване ім’я
admin_rename_save = Змінити ім’я користувача
admin_detail_title = Відомості про обліковий запис
admin_detail_invite = Запрошення { $code }
admin_detail_posts = Дописи
admin_detail_followers = Читачі
admin_detail_following = Відстежується
admin_detail_open_reports = Відкриті скарги
admin_detail_sessions = Сеанси
admin_detail_passkeys = Ключі доступу
admin_detail_tokens = Токени API
admin_detail_joined = Приєднався(-лася) { $time }
admin_detail_recent_posts = Нещодавні дописи
admin_detail_no_posts = Дописів немає.
admin_detail_reports = Скарги на цей обліковий запис
admin_detail_no_reports = Скарг немає.
admin_detail_history = Історія модерації
admin_detail_nothing = Поки нічого.
admin_instance_title = Інстанс
admin_instance_name = Назва інстансу
admin_instance_description = Опис
admin_instance_contact = Електронна пошта для зв’язку
admin_instance_contact_hint = Показується на сторінках Умов і Конфіденційності
admin_instance_min_age = Мінімальний вік
admin_instance_rules = Правила
admin_instance_rules_hint = Одне правило на рядок
admin_instance_preview = Переглянути умови
admin_instance_save = Зберегти
admin_instance_saved = Налаштування інстансу збережено
admin_instance_template_note = Умови використання та Політика конфіденційності є шаблонами, що використовують ці значення. Перш ніж відкривати інстанс для всіх, нехай юрист перевірить їх на відповідність вашій юрисдикції.
admin_signups_title = Реєстрація
admin_signups_desc = Хто може створювати обліковий запис на цьому інстансі
admin_signups_open = Відкрита
admin_signups_open_hint = Будь-хто може створити обліковий запис.
admin_signups_invite = Лише за запрошенням
admin_signups_invite_hint = Потрібен код запрошення з вкладки «Запрошення».
admin_signups_closed = Закрита
admin_signups_closed_hint = Ніхто не може зареєструватися.
admin_filter_title = Фільтр вмісту
admin_filter_desc = Нічого не блокується, доки ви не додасте слова нижче. Заблоковані слова діють для імен користувачів, відображуваних імен, місць, описів, дописів і назв списків, зокрема для замаскованих написань. Зарезервовані імена на кшталт «admin» зареєструвати не можна.
admin_filter_blocked = Заблоковані слова
admin_filter_blocked_hint = По одному на рядок, шукаються всередині слів
admin_filter_allowed = Дозволені слова
admin_filter_allowed_hint = Винятки із заблокованих слів, по одному на рядок
admin_filter_reserved = Додаткові зарезервовані імена
admin_filter_reserved_hint = По одному на рядок
admin_filter_save = Зберегти фільтр
admin_filter_saved = Фільтр збережено
admin_filter_field_handle = ім’я користувача
admin_filter_field_name = ім’я
admin_filter_field_location = місце
admin_filter_field_bio = опис
admin_scan_title = Сканувати облікові записи
admin_scan_desc = Знайти облікові записи, у яких ім’я користувача, ім’я чи опис порушують фільтр.
admin_scan_run = Сканувати зараз
admin_scan_clean = Жоден обліковий запис не порушує фільтр.
admin_scan_found =
    { $count ->
        [one] { $count } обліковий запис потребує уваги
        [few] { $count } облікові записи потребують уваги
        [many] { $count } облікових записів потребують уваги
       *[other] { $count } облікового запису потребує уваги
    }
admin_scan_reason = Поле «{ $field }» містить { $reason }
admin_ann_title = Нове оголошення
admin_ann_desc = Банер угорі екрана в усіх, доки він не спливе або ви його не завершите. Люди можуть його закрити.
admin_ann_text = Повідомлення
admin_ann_link = Посилання
admin_ann_link_hint = Необов’язково: https://… або /шлях
admin_ann_tone = Стиль
admin_ann_tone_info = Інформація
admin_ann_tone_success = Гарні новини
admin_ann_tone_warning = Попередження
admin_ann_duration = Тривалість
admin_ann_days =
    { $count ->
        [one] { $count } день
        [few] { $count } дні
        [many] { $count } днів
       *[other] { $count } дня
    }
admin_ann_week = 1 тиждень
admin_ann_until_ended = До завершення
admin_ann_publish = Опублікувати
admin_ann_published = Оголошення опубліковано
admin_ann_list = Усі оголошення
admin_ann_none = Оголошень поки немає.
admin_ann_end = Завершити
admin_ann_ended = завершено
admin_ann_expired = спливло
admin_ann_inactive = Не показується
admin_ann_until = до { $date }
admin_inv_title = Коди запрошень
admin_inv_desc = Дозвольте приєднатися конкретним людям, поки реєстрація лише за запрошенням.
admin_inv_note = Примітка (для кого)
admin_inv_uses = Використань
admin_inv_use_count =
    { $count ->
        [one] { $count } використання
        [few] { $count } використання
        [many] { $count } використань
       *[other] { $count } використання
    }
admin_inv_expiry = Спливає
admin_inv_week = Через 1 тиждень
admin_inv_days =
    { $count ->
        [one] Через { $count } день
        [few] Через { $count } дні
        [many] Через { $count } днів
       *[other] Через { $count } дня
    }
admin_inv_never = Ніколи
admin_inv_create = Створити код
admin_inv_created = Код { $code } створено
admin_inv_created_copied = Код { $code } створено й скопійовано
admin_inv_list = Коди
admin_inv_none = Кодів запрошень поки немає.
admin_inv_state_active = Активний
admin_inv_state_revoked = Відкликаний
admin_inv_state_used = Вичерпаний
admin_inv_state_expired = Прострочений
admin_inv_used = Використано { $uses } із { $max }
admin_inv_expires = спливає { $date }
admin_inv_copy = Копіювати код { $code }
admin_inv_copied = Код скопійовано
admin_inv_copy_failed = Не вдалося скопіювати. Код: { $code }.
admin_inv_revoke = Відкликати
admin_inv_revoke_title = Відкликати це запрошення?
admin_inv_revoke_body = Люди, які вже приєдналися за ним, зберігають свої облікові записи.
settings_devices = Пристрої та сеанси
settings_devices_desc = Перегляньте, де ви увійшли, і вийдіть віддалено
settings_passkeys = Ключі доступу
settings_passkeys_desc = Входьте за відбитком пальця, обличчям, екраном блокування або ключем безпеки
settings_affiliation = Афіліація
settings_affiliation_desc = Організації, до яких ви належите, або облікові записи вашої організації
settings_bots = Боти та API
settings_bots_desc = API-токени та позначка «Автоматизований»
settings_saved = Збережено
passkey_title = Ваші ключі доступу
passkey_desc = Входьте за відбитком пальця, обличчям, екраном блокування чи ключем безпеки замість пароля. Ключі доступу не можна виманити фішингом, а на нашому сервері не зберігається нічого секретного.
passkey_none = Ключів доступу поки немає.
passkey_add = Додати ключ доступу
passkey_added = Ключ доступу «{ $name }» додано
passkey_default_name = Ключ доступу
passkey_added_ago = додано { $time }
passkey_used_ago = востаннє використано { $time }
passkey_never_used = ще не використовувався
passkey_rename = Перейменувати
passkey_rename_label = Перейменувати ключ доступу { $name }
passkey_rename_title = Перейменувати ключ доступу
passkey_name = Назва
passkey_rename_save = Зберегти
passkey_remove = Вилучити
passkey_remove_label = Вилучити ключ доступу { $name }
passkey_remove_title = Вилучити «{ $name }»?
passkey_remove_body = Ви більше не зможете входити за допомогою цього ключа доступу. Можливо, вам також варто видалити його зі свого пристрою чи менеджера паролів.
passkey_removed = Ключ доступу вилучено
passkey_unsupported = Цей браузер не може використовувати ключі доступу на цій адресі.
passkey_needs_domain = Для ключів доступу сайт має мати доменне ім’я, а не IP-адресу.
passkey_already_here = На цьому пристрої вже є ключ доступу для цього облікового запису.
token_scope_read = Читання
token_scope_write = Публікувати й взаємодіяти
token_scope_messages = Особисті повідомлення
token_scope_read_hint = Стрічки, сповіщення, профілі, пошук і потік подій у реальному часі
token_scope_write_hint = Публікація, видалення, вподобання, поширення, закладки, підписки та завантаження
token_scope_messages_hint = Читання й надсилання особистих повідомлень
token_title = Токени API
token_desc = Токени дозволяють скриптам і ботам користуватися вашим обліковим записом через API. Бережіть їх як паролі: власник токена може діяти від вашого імені в межах його прав.
token_none = Токенів поки немає.
token_name = Назва токена
token_name_hint = Наприклад: бот погоди
token_scopes = Дозволи
token_create = Створити токен
token_used_ago = востаннє використано { $time }
token_never_used = ще не використовувався
token_revoke = Відкликати
token_revoke_label = Відкликати токен { $name }
token_revoke_title = Відкликати «{ $name }»?
token_revoke_body = Усе, що ним користується, перестане працювати.
token_revoked = Токен відкликано
token_secret_title = Ваш новий токен
token_secret_warning = Скопіюйте його зараз. З міркувань безпеки його більше не буде показано.
token_secret_label = Новий API-токен
token_copy = Копіювати
token_copied = Токен скопійовано
token_copy_failed = Не вдалося скопіювати. Виділіть токен і скопіюйте вручну.
token_done = Готово
bot_title = Автоматизований обліковий запис
bot_desc = Показує позначку «Автоматизований» біля імені, щоб усі знали, що публікує бот.
bot_switch = Це автоматизований обліковий запис
bot_owner = Керує
bot_owner_hint = Ім’я користувача людини або організації, що стоїть за ботом. Необов’язково.
bot_owner_save = Зберегти
bot_docs = Документація для розробників
bot_docs_desc = Кінцеві точки, події в реальному часі та приклади ботів
aff_yours_title = Ваша афіліація
aff_yours_desc = Організація може запросити вас показувати її значок біля вашого імені. Нічого не зміниться, доки ви не погодитеся, і ви можете вийти будь-коли.
aff_none = Ви не афілійовані з жодною організацією.
aff_invites_title = Запрошення
aff_accept = Прийняти
aff_decline = Відхилити
aff_leave = Вийти
aff_leave_title = Вийти з цієї організації?
aff_leave_body = Її значок зникне з вашого профілю.
aff_remove_title = Вилучити { $handle }?
aff_remove_body = Ваш значок зникне з профілю цієї людини.
aff_remove = Вилучити
aff_remove_label = Вилучити { $handle }
aff_ended = Афіліацію завершено
aff_declined = Запрошення відхилено
aff_accepted = Афіліацію додано
aff_invited = Запрошення надіслано
aff_org_title = Афілійовані облікові записи
aff_org_desc = Запрошуйте людей, щоб ваш значок показувався поруч з їхнім ім’ям. Вони мають прийняти запрошення.
aff_handle = Ім’я користувача
aff_label = Позначка
aff_label_hint = Необов’язково, наприклад «Інженер»
aff_invite = Надіслати запрошення
aff_members_none = Афілійованих поки немає.
aff_pending = Запрошено
aff_not_org = Ви організація? Попросіть адміністратора цього інстансу позначити ваш обліковий запис.
auth_passkey = Увійти за допомогою ключа доступу
auth_or = або
auth_invite = Код запрошення
auth_invite_hint = Цей інстанс приймає нових людей лише за запрошенням.
auth_invite_required = Введіть код запрошення, який вам надали.
landing_developers = Розробникам
landing_invite = Реєстрація лише за запрошенням. Маєте код? Створіть обліковий запис.
dev_title = Розробникам
dev_hero = Боти та API
dev_hero_sub = Створюйте ботів, скрипти та інтеграції для { $name }
dev_intro_h = Створюйте для { $name }
dev_intro_p = Усе, що роблять застосунки tuita, проходить через невеликий JSON API, і боти та скрипти теж можуть ним користуватися. Потрібні обліковий запис на { $name } і особистий API-токен. Реєструвати застосунок і проходити OAuth не треба.
dev_intro_1 = Створіть обліковий запис для бота або використайте свій.
dev_intro_2 = У розділі «Налаштування → Боти та API» увімкніть «Це автоматизований обліковий запис», щоб люди бачили позначку «Автоматизований», і вкажіть, хто ним керує.
dev_intro_3 = Створіть токен із потрібними дозволами й тримайте його в таємниці.
dev_auth_h = Автентифікація
dev_auth_p = Надсилайте токен у заголовку Authorization. Усі запити й відповіді мають формат JSON (UTF-8), крім завантажень, які надсилаються як multipart.
dev_auth_prefix = Токен починається з tuita_ і далі має 43 літери, цифри, дефіси та підкреслення. Зберігається лише хеш, тому втрачений токен показати знову не можна: створіть новий. Токен має один або кілька таких дозволів:
dev_auth_refused = Токен не може змінювати паролі, сеанси, токени чи ключі доступу, не може експортувати або видаляти обліковий запис і не може використовувати API адміністратора. Для цього потрібен справжній вхід на вебсайті; відповідь буде 403.
dev_auth_errors = Помилки повертаються як JSON із повідомленням error і статусом 4xx. 401 означає, що токен хибний або відкликаний, 403 — бракує дозволу або маршрут заборонено, а 429 надходить із заголовком Retry-After.
dev_post_h = Ваш перший допис
dev_post_p = Спершу завантажте зображення й відео через POST /api/v1/media (multipart), а потім прикріпіть їх за id у mediaIds. Відповідь задає replyTo, цитата задає quoteOf.
dev_endpoints_h = Кінцеві точки
dev_endpoints_intro = Шляхи у фігурних дужках є заповнювачами. Списки розбиваються на сторінки через ?before= зі значенням next попередньої сторінки; limit від 1 до 40. У стовпці «Дозвіл» указано дозвіл, потрібний токену.
dev_col_method = Метод
dev_col_path = Шлях
dev_col_scope = Дозвіл
dev_col_what = Що робить
dev_ep_me = Обліковий запис, якому належить токен
dev_ep_patch_me = Змінити профіль і налаштування цього облікового запису
dev_ep_bot = Увімкнути чи вимкнути позначку «Автоматизований» і вказати, хто керує ботом
dev_ep_following = Домашня стрічка людей, за якими ви стежите
dev_ep_foryou = Стрічка «Для вас»; посторінково через ?cursor=
dev_ep_notifications = Сповіщення (?filter=mentions для відповідей, згадок і цитувань)
dev_ep_post = Допис із його гілкою та відповідями
dev_ep_user = Профіль (handle@domain для облікових записів на інших інстансах)
dev_ep_user_posts = Дописи людини (?tab=replies, media або likes; для media також ?kind=photo або video)
dev_ep_search = Пошук дописів, людей чи медіа з операторами на кшталт from: і has:media
dev_ep_trends = Найпопулярніші теги за останній тиждень
dev_ep_stream = Події в реальному часі (Server-Sent Events)
dev_ep_create = Написати допис
dev_ep_thread = Написати від 2 до 25 дописів як ланцюжок
dev_ep_delete = Видалити один зі своїх дописів
dev_ep_like = Вподобати допис (DELETE, щоб скасувати). Також /repost і /bookmark
dev_ep_follow = Стежити за кимось (DELETE, щоб перестати)
dev_ep_media = Завантажити зображення чи відео як multipart; у відповіді буде id для прикріплення
dev_ep_schedule = Запланувати допис на потім
dev_ep_report = Поскаржитися модераторам на допис чи обліковий запис
dev_ep_conversations = Ваші розмови в особистих повідомленнях
dev_ep_message = Надіслати особисте повідомлення; тіло є JSON із полем text
dev_realtime_h = Події в реальному часі
dev_realtime_p = GET /api/v1/stream — це потік Server-Sent Events. Він надсилає подію notification для кожного вподобання, поширення, підписки, відповіді, згадки чи цитування, подію message для особистих повідомлень і значки з кількістю непрочитаних. Кожні 25 секунд надсилається рядок-коментар, а клієнтам пропонується повторити спробу через 5 секунд; перепідключіться, якщо з’єднання обірветься.
dev_example_h = Приклад: бот, що відповідає на згадки
dev_example_node = Node 22 або новіша, без пакетів:
dev_example_python = Те саме на Python, лише з requests:
dev_limits_h = Обмеження
dev_limit_rate = Кожен токен може робити 600 запитів за 10 хвилин. Після перевищення ліміту сервер відповідає 429 із заголовком Retry-After.
dev_limit_tokens = Обліковий запис може мати до 20 токенів і створювати до 20 токенів за годину.
dev_limit_posting = Для публікації, підписок, скарг та інших дій діють власні обмеження на обліковий запис.
dev_rules_h = Правила для ботів
dev_rule_1 = Увімкніть позначку «Автоматизований» і вкажіть, хто керує ботом.
dev_rule_2 = Відповідайте чи пишіть лише тим, хто цього просив: згадав бота, стежить за ним або першим написав йому. Не надсилайте небажаних згадок чи особистих повідомлень.
dev_rule_3 = Не поспішайте: залишайтеся значно нижче наведених обмежень.
dev_rule_4 = Дотримуйтеся Умов і правил інстансу. Модератори можуть призупиняти ботів, як і будь-який обліковий запис.
admin_act_log_admin_bootstrap = Налаштовано першого адміністратора
admin_act_log_announcement_create = Оголошення опубліковано
admin_act_log_announcement_end = Оголошення завершено
admin_act_log_instance_settings = Налаштування інстансу змінено
admin_act_log_invite_create = Запрошення створено
admin_act_log_invite_revoke = Запрошення відкликано
admin_act_log_post_delete = Допис видалено
admin_act_log_report_dismiss = Скаргу закрито
admin_act_log_user_delete = Обліковий запис видалено
admin_act_log_user_org = Позначено як організацію
admin_act_log_user_unorg = Статус організації знято
admin_act_log_user_purge_posts = Усі дописи видалено
admin_act_log_user_rename = Обліковий запис перейменовано
admin_act_log_user_role = Роль змінено
admin_act_log_user_suspend = Обліковий запис призупинено
admin_act_log_user_unsuspend = Обліковий запис відновлено
admin_act_log_user_unverify = Підтвердження знято
admin_act_log_user_verify = Обліковий запис підтверджено
admin_act_log_domain_block = Інстанс заблоковано
admin_act_log_domain_unblock = Інстанс розблоковано
admin_act_log_federation_retry = Запущено повторну доставку
admin_fed_stat_domains = Відомі інстанси
admin_fed_stat_domains_sub = помічені цим інстансом
admin_fed_stat_accounts = Віддалені облікові записи
admin_fed_stat_accounts_sub = відомі з інших місць
admin_fed_stat_posts = Віддалені дописи
admin_fed_stat_posts_sub = збережені локально
admin_fed_stat_queue = У черзі
admin_fed_stat_queue_sub = доставок очікує
admin_fed_queue_title = Черга доставки
admin_fed_queue_desc = Підписані активності, що чекають на доставку іншим інстансам. Невдалі повторюються зі зростаючими паузами й відкидаються після 12 спроб.
admin_fed_queue_empty = Нічого не очікує. Усі доставки надіслано.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } доставка в черзі
        [few] { $compact } доставки в черзі
        [many] { $compact } доставок у черзі
       *[other] { $compact } доставки в черзі
    }
admin_fed_queue_due = Готові до надсилання: { $count }
admin_fed_queue_oldest = Найстаріша в черзі: { $date }
admin_fed_retry = Повторити зараз
admin_fed_retry_done =
    { $count ->
        [one] { $compact } доставку буде повторено зараз.
        [few] { $compact } доставки буде повторено зараз.
        [many] { $compact } доставок буде повторено зараз.
       *[other] { $compact } доставки буде повторено зараз.
    }
admin_fed_failing = Інстанси зі збоями
admin_fed_failing_none = Жоден інстанс не дає збоїв.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } доставка
        [few] { $compact } доставки
        [many] { $compact } доставок
       *[other] { $compact } доставки
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } спроба
        [few] { $compact } спроби
        [many] { $compact } спроб
       *[other] { $compact } спроби
    }
admin_fed_failing_next = наступна спроба { $date }
admin_fed_failing_error = Остання помилка
admin_fed_failing_no_error = Помилок не записано
admin_fed_actor_title = Актор інстансу
admin_fed_actor_desc = Інші інстанси звіряють із цим ключем запити, які підписує цей інстанс.
admin_fed_actor_id = Актор
admin_fed_actor_key = ID ключа
admin_fed_actor_fingerprint = Відбиток SHA-256
admin_fed_actor_copy = Копіювати відбиток
admin_fed_actor_copied = Відбиток скопійовано
admin_fed_actor_copy_failed = Не вдалося скопіювати. Відбиток: { $fingerprint }
admin_fed_blocks_title = Заблоковані інстанси
admin_fed_blocks_desc = Обмеження приховує облікові записи інстансу з публічних стрічок і пошуку. Блокування приховує їх усюди, відкидає все, що вони надсилають, і припиняє доставку їм.
admin_fed_domain = Домен
admin_fed_domain_hint = example.social або посилання на будь-яку його сторінку
admin_fed_severity = Суворість
admin_fed_silence = Обмежити
admin_fed_suspend = Заблокувати
admin_fed_reason = Причина
admin_fed_reason_hint = Необов’язково, бачить лише команда
admin_fed_block = Заблокувати інстанс
admin_fed_blocked = { $domain } заблоковано
admin_fed_none = Жоден інстанс не заблоковано.
admin_fed_list = Блокування
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } відомий обліковий запис
        [few] { $compact } відомі облікові записи
        [many] { $compact } відомих облікових записів
       *[other] { $compact } відомого облікового запису
    }
admin_fed_row_no_reason = Причину не вказано
admin_fed_row_changed = Для { $domain } тепер встановлено: { $severity }
admin_fed_remove = Вилучити
admin_fed_remove_label = Зняти блокування з { $domain }
admin_fed_remove_title = Зняти блокування з { $domain }?
admin_fed_remove_body = Його облікові записи й дописи повернуться, а доставка до нього відновиться.
admin_fed_removed = { $domain } більше не заблоковано
remote_domain_title = Обліковий запис на { $domain }
remote_view_original = Переглянути на { $domain }
visibility_unlisted = Не в списках
visibility_unlisted_hint = Не показується в публічних стрічках
visibility_followers = Лише читачі
visibility_followers_hint = Бачити це можуть лише читачі
visibility_direct = Лише згадані
visibility_direct_hint = Бачити це можуть лише згадані люди
follow_requested = Запит надіслано
follow_cancel_request = Скасувати запит на підписку
follow_requested_toast = Запит на підписку надіслано { $handle }
follow_cancelled_toast = Запит на підписку скасовано
profile_locked = Схвалює читачів
profile_locked_hint = Запити на підписку потребують схвалення цього облікового запису.
profile_original = Переглянути профіль на { $domain }
profile_remote_note = Копія з { $domain }. Лічильники й дописи — це те, що бачив цей інстанс.
remote_lookup_loading = Пошук { $handle } на { $domain }…
remote_lookup_notfound_title = Такого облікового запису немає
remote_lookup_notfound_body = { $handle } не існує, або { $domain } не ділиться ним.
remote_lookup_failed_title = Не вдалося завантажити { $handle }
explore_lookup = Пошук { $target }…
explore_lookup_empty_body = Цей обліковий запис чи допис не знайдено. Інший інстанс може бути вимкнений або не ділитися ним.
language_label = Мова
oauth_title = Авторизація застосунку
oauth_loading = Перевірка запиту…
oauth_wants = Цей застосунок хоче користуватися вашим обліковим записом { $handle }.
oauth_name_warning = Будь-хто може вибрати будь-яку назву застосунку. Схвалюйте лише застосунки, яким довіряєте.
oauth_website = Вебсайт
oauth_allow = Він зможе
oauth_scope_read = Читати ваш профіль, дописи, підписки, списки, стрічки та сповіщення, а також особисті повідомлення без наскрізного шифрування.
oauth_scope_write = Публікувати, редагувати й видаляти від вашого імені, вподобувати, стежити, блокувати, приглушувати та змінювати профіль і налаштування.
oauth_scope_messages = Читати й надсилати ваші особисті повідомлення.
oauth_push_note = Push-сповіщення не підтримуються. Застосунок, який їх запитує, їх не отримає.
oauth_redirect = Після вашої відповіді вас буде перенаправлено на
oauth_redirect_oob = Застосунок не отримує перенаправлення. Тут буде показано код, який потрібно вставити в нього.
oauth_approve = Авторизувати
oauth_deny = Відхилити
oauth_switch = Використати інший обліковий запис
oauth_sent_approved = Авторизовано. Повертаємо вас на { $host }. Якщо нічого не відбувається, закрийте цю вкладку.
oauth_sent_denied = Відхилено. Повертаємо вас на { $host }. Якщо нічого не відбувається, закрийте цю вкладку.
oauth_denied = Відхилено. Застосунку нічого не передано.
oauth_code_title = Код авторизації
oauth_code_body = Вставте цей код у застосунок. Він діє один раз і спливає через 10 хвилин.
oauth_code_copy = Копіювати код
oauth_code_copied = Код скопійовано
oauth_code_copy_failed = Не вдалося скопіювати. Виділіть код і скопіюйте вручну.
oauth_error_title = Цей запит не можна авторизувати
oauth_error_note = Застосунку нічого не надіслано.
oauth_error_invalid_request = Запит хибно сформовано, або він просить повернутися на адресу, яку застосунок не зареєстрував. Поверніться до застосунку й спробуйте ще раз.
oauth_error_invalid_client = Цей застосунок тут не зареєстровано.
oauth_error_invalid_scope = Застосунок просить доступ, якого tuita не надає, або більший, ніж зареєстровано.
oauth_error_unsupported_response_type = Застосунок просить тип входу, який tuita не підтримує.
oauth_error_rate = Ви відповіли на забагато запитів. Зачекайте кілька хвилин і спробуйте ще раз.
oauth_error_unreachable = Не вдалося зв’язатися з tuita. Перевірте з’єднання й спробуйте ще раз.
oauth_home = Перейти до tuita
share_send_chat = Надіслати в чаті
share_download = Завантажити медіа
share_download_n = Завантажити медіа { $number }
share_chat_title = Надіслати допис у чаті
share_chat_sent = Надіслано { $handle }
hover_card_label = Картка профілю: { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Тип медіа
profile_media_all = Усі медіа
profile_media_photos = Фото
profile_media_videos = Відео
profile_empty_videos_title = Ще немає відео
profile_empty_videos_body = Тут з’являтимуться дописи з відео.
account_add_existing = Додати наявний обліковий запис
rail_label = Пошук і пропозиції
rail_search_submit = Пошук
rail_show_more = Показати більше
rail_footer = Посилання
chat_new = Новий чат
chat_search = Шукати розмови
chat_search_none = Жодна розмова не відповідає пошуку.
chat_start_title = Почати розмову
chat_start_body = Виберіть розмову зі списку або почніть нову.
explore_sections = Розділи огляду
explore_see_more = Показати більше
history_title = Історія
history_tabs = Розділи історії
history_likes_label = Вподобані дописи
video_play = Відтворити
video_pause = Пауза
video_mute = Вимкнути звук
video_unmute = Увімкнути звук
video_volume = Гучність
video_seek = Перемотування
video_position = { $current } із { $total }
video_speed = Швидкість відтворення
video_fullscreen = На весь екран
video_exit_fullscreen = Вийти з повноекранного режиму
video_pip = Картинка в картинці
video_exit_pip = Вийти з режиму «картинка в картинці»
video_error = Це відео неможливо відтворити.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Сховати панель
lightbox_show_panel = Показати панель
lightbox_view_post = Переглянути допис
lightbox_panel = Допис і відповіді
emotepick_tabs = Набори емодзі
emotepick_tab_emoji = Емодзі
emotepick_tab_custom = Власні
admin_emote_tab = Смайли
admin_emote_title = Власні смайли
admin_emote_desc = Смайли належать усьому інстансу. Будь-хто може використовувати їх як :shortcode: у дописах, іменах і повідомленнях. Код смайла не можна змінити; видаліть смайл і завантажте його знову.
admin_emote_shortcode = Код
admin_emote_shortcode_hint = Від 2 до 32 літер, цифр або підкреслень. Показується як :shortcode:.
admin_emote_invalid = Використовуйте лише літери, цифри та підкреслення, від 2 до 32 символів.
admin_emote_category = Категорія
admin_emote_category_hint = Необов’язково. Групує смайли в палітрі.
admin_emote_file = Зображення смайла
admin_emote_file_hint = PNG, JPEG, WebP або GIF, до 256 КБ. Перетягніть файл сюди або виберіть його.
admin_emote_file_type = Цей файл не є зображенням PNG, JPEG, WebP чи GIF.
admin_emote_file_size = Зображення завелике (макс. 256 КБ).
admin_emote_choose_file = Вибрати зображення
admin_emote_change_file = Змінити зображення
admin_emote_upload = Завантажити смайл
admin_emote_upload_failed = Не вдалося завантажити смайл. Спробуйте ще раз.
admin_emote_uploaded = Додано :{ $shortcode }:.
admin_emote_list = Смайли цього інстансу
admin_emote_none = Власних смайлів ще немає.
admin_emote_no_category = Без категорії
admin_emote_category_for = Категорія :{ $shortcode }:
admin_emote_saved = Категорію збережено.
admin_emote_delete = Видалити :{ $shortcode }:
admin_emote_delete_title = Видалити цей смайл?
admin_emote_delete_body = :{ $shortcode }: більше не відображається зображенням ніде й показується як звичайний текст. Дописи, що його використовували, зберігають текст.
admin_emote_delete_confirm = Видалити смайл
admin_emote_deleted = Смайл видалено.
admin_act_log_emoji_create = Смайл додано
admin_act_log_emoji_update = Смайл змінено
admin_act_log_emoji_delete = Смайл видалено
e2ee_settings = Зашифровані повідомлення
e2ee_settings_desc = Парольна фраза, цей пристрій і скидання
e2ee_status_label = Стан
e2ee_status_none = Вимкнено. Увімкніть, щоб надсилати й читати повідомлення з наскрізним шифруванням.
e2ee_status_locked = Заблоковано на цьому пристрої. Введіть парольну фразу, щоб читати тут свої повідомлення.
e2ee_status_ready = Увімкнено й розблоковано на цьому пристрої.
e2ee_keys_error = Не вдалося перевірити ваші ключі шифрування.
e2ee_setup_title = Увімкнути зашифровані повідомлення
e2ee_setup_body = Повідомлення людям на цьому сервері шифруються у вашому браузері. Сервер зберігає лише текст, який не може прочитати. Парольна фраза дає змогу розблокувати повідомлення на іншому пристрої.
e2ee_setup_warning = Парольну фразу не можна відновити. Якщо ви її забудете, можна скинути шифрування, але попередні повідомлення буде втрачено.
e2ee_setup_submit = Увімкнути
e2ee_setup_done = Зашифровані повідомлення ввімкнено
e2ee_unlock_title = Розблокувати зашифровані повідомлення
e2ee_unlock_body = Введіть парольну фразу, щоб читати й надсилати зашифровані повідомлення на цьому пристрої.
e2ee_unlock_submit = Розблокувати
e2ee_unlock_wrong = Ця парольна фраза хибна.
e2ee_unlock_done = Розблоковано
e2ee_passphrase = Парольна фраза
e2ee_passphrase_confirm = Підтвердьте парольну фразу
e2ee_passphrase_hint = Щонайменше 10 символів.
e2ee_passphrase_short = Використайте щонайменше 10 символів.
e2ee_passphrase_mismatch = Парольні фрази не збігаються.
e2ee_failed = Щось пішло не так. Спробуйте ще раз.
e2ee_unsupported = Цей браузер не підтримує зашифровані повідомлення.
e2ee_forget = Забути на цьому пристрої
e2ee_forget_hint = Видаляє з цього браузера ваш ключ і контакти, яким ви довіряєте. Парольна фраза знову їх розблокує.
e2ee_forget_done = Забуто на цьому пристрої
e2ee_reset = Скинути шифрування
e2ee_reset_hint = Створює новий ключ. Повідомлення, надіслані й отримані раніше, стануть нечитабельними.
e2ee_reset_title = Скинути шифрування?
e2ee_reset_body = Попередні повідомлення стануть нечитабельними, для вас і для людей, яким ви писали, і їх не можна повернути. Ваших контактів попросять довіряти новому ключу. Виберіть нову парольну фразу.
e2ee_reset_submit = Скинути
e2ee_reset_done = Шифрування скинуто
e2ee_note_sealed = Наскрізне шифрування
e2ee_note_remote = Без наскрізного шифрування: ця людина на іншому сервері.
e2ee_note_plain = Без наскрізного шифрування.
e2ee_plain_mark = Не зашифровано
e2ee_preview = Зашифроване повідомлення
e2ee_line_sealed = Зашифроване повідомлення
e2ee_line_own_key = Надіслано на ключ, якого у вас більше немає
e2ee_line_peer_key = Надіслано ключем, якому ви ще не довіряєте
e2ee_line_unreadable = Не вдалося відкрити
e2ee_line_unsupported = Цей браузер не може відкривати зашифровані повідомлення
e2ee_block_peer_off = Користувач { $handle } ще не ввімкнув зашифровані повідомлення.
e2ee_block_self_off = Спершу ввімкніть зашифровані повідомлення у Налаштуваннях.
e2ee_block_unlock = Спершу розблокуйте зашифровані повідомлення у Налаштуваннях.
e2ee_block_downgrade = Користувач { $handle } уже використовував зашифровані повідомлення. Ця розмова не повернеться до незашифрованих повідомлень.
e2ee_block_changed = Довіртеся новому ключу користувача { $handle }, щоб писати цій людині.
e2ee_block_error = Не вдалося перевірити ваші ключі шифрування.
e2ee_changed_title = Ключ шифрування користувача { $name } змінився
e2ee_changed_body = Ця людина могла скинути шифрування чи перейти на новий пристрій, або хтось вклинився між вами. Перш ніж довіряти, порівняйте з нею новий номер безпеки. Доки цього не зробите, надсилати не можна.
e2ee_changed_old = Номер безпеки, якому ви довіряли
e2ee_changed_new = Новий номер безпеки
e2ee_trust = Довіряти новому ключу
e2ee_safety_open = Номер безпеки
e2ee_safety_title = Номер безпеки
e2ee_safety_body = Порівняйте цей номер із { $name } особисто чи під час дзвінка, не в цьому чаті. Якщо він збігається на обох екранах, між вами нікого немає.

## 0031
composer_upload_failed = Не вдалося завантажити. Спробуйте ще раз.
composer_send_failed = Не вдалося зʼєднатися з tuita. Перевірте зʼєднання й спробуйте ще раз.
composer_open_failed = Не вдалося відкрити редактор допису. Спробуйте ще раз.
