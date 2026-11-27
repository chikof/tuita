app_name = tuita
loading = Wczytywanie
close = Zamknij
dismiss = Odrzuć
notifications = Powiadomienia
avatar_more = jeszcze { $count }
char_counter_left =
    { $count ->
        [one] { $count } znak pozostał
        [few] { $count } znaki pozostały
        [many] { $count } znaków pozostało
       *[other] { $count } znaku pozostało
    }
char_counter_over =
    { $count ->
        [one] { $count } znak ponad limit
        [few] { $count } znaki ponad limit
        [many] { $count } znaków ponad limit
       *[other] { $count } znaku ponad limit
    }
not_found_title = Nic tu nie ma
not_found_body = Ta strona nie istnieje albo została przeniesiona.
not_found_back = Wróć
tagline = Każdy wpis w dwóch kolorach tuszu.
error_title = Coś poszło nie tak
cancel = Anuluj
retry = Spróbuj ponownie
back = Wstecz
view = Zobacz
follow = Obserwuj
following = Obserwowani
follows_you = Obserwuje Cię
verified = Zweryfikowane konto
bot_account = Konto automatyczne
bot_label = Bot
link_copied = Skopiowano link do schowka
text_copied = Skopiowano tekst
time_now = teraz
time_minutes = { $n } min
time_hours = { $n } godz.
time_days = { $n } d
nav_main = Główne
nav_home = Główna
nav_explore = Odkrywaj
nav_notifications = Powiadomienia
nav_bookmarks = Zakładki
nav_profile = Profil
nav_settings = Ustawienia
nav_compose = Opublikuj
nav_menu = Menu
badge_unread = Nieprzeczytane: { $count }
account_menu = Konta
account_switch = Przełącz konto
account_switch_to = Przełącz na { $handle }
account_add = Dodaj konto
account_logout = Wyloguj { $handle }
account_logout_all = Wyloguj ze wszystkich kont
home_feed = Oś czasu
home_for_you = Dla Ciebie
home_following = Obserwowani
home_empty_title = Jeszcze nic tu nie ma
home_empty_body = Bądź pierwszy(-a), kto coś opublikuje.
home_following_empty_title = W Twoim gronie jest cicho
home_following_empty_body = Obserwuj osoby, aby wypełnić tę oś czasu ich wpisami i podaniami dalej.
home_find_people = Znajdź ludzi
new_posts =
    { $count ->
        [one] Pokaż { $count } nowy wpis
        [few] Pokaż { $count } nowe wpisy
        [many] Pokaż { $count } nowych wpisów
       *[other] Pokaż { $count } nowego wpisu
    }
new_posts_many = Pokaż ponad 99 nowych wpisów
composer_title = Nowy wpis
composer_reply_title = Odpowiedz
composer_quote_title = Cytuj wpis
composer_placeholder = Co się dzieje?
composer_reply_placeholder = Opublikuj odpowiedź
composer_text_label = Treść wpisu
composer_replying_to = Odpowiedź dla { $handle }
composer_cw_label = Ostrzeżenie o treści
composer_cw_placeholder = Ostrzeżenie o treści, np. spoilery
composer_cw_toggle = Dodaj ostrzeżenie o treści
composer_sensitive_toggle = Oznacz multimedia jako wrażliwe
composer_options = Opcje wpisu
composer_policy_label = Kto może odpowiadać
composer_suggestions = Podpowiedzi
composer_tag_posts =
    { $count ->
        [one] { $compact } wpis
        [few] { $compact } wpisy
        [many] { $compact } wpisów
       *[other] { $compact } wpisu
    }
composer_post = Opublikuj
composer_reply = Odpowiedz
policy_everyone = Każdy może odpowiedzieć
policy_following = Odpowiadać mogą obserwowane przez Ciebie osoby
policy_mentioned = Odpowiadać mogą tylko wspomniane przez Ciebie osoby
post_sent = Wysłano wpis
post_by = Wpis od { $name }
post_reposted_by = { $name } podaje dalej
post_pinned = Przypięty wpis
post_replying_to = Odpowiedź dla { $handle }
post_replying_unavailable = Odpowiedź na niedostępny wpis
post_unavailable = Ten wpis jest niedostępny.
post_menu = Więcej
post_copy_link = Kopiuj link
post_copy_text = Kopiuj tekst
post_pin = Przypnij do profilu
post_unpin = Odepnij od profilu
post_pinned_toast = Przypięto do Twojego profilu
post_unpinned = Odpięto od Twojego profilu
post_delete = Usuń wpis
post_delete_title = Usunąć wpis?
post_delete_body = Tej operacji nie można cofnąć. Wpis zniknie z Twojego profilu, z osi czasu osób, które Cię obserwują, i z wyszukiwania.
post_deleted = Usunięto wpis
post_view_profile = Zobacz { $handle }
post_mute = Wycisz { $handle }
post_block = Zablokuj { $handle }
post_reply_label = Odpowiedz ({ $count })
post_reply_following_only = Odpowiadać mogą tylko osoby obserwowane przez autora.
post_reply_mentioned_only = Odpowiadać mogą tylko osoby wspomniane w tym wpisie.
post_repost_menu = Opcje podania dalej
post_repost = Podaj dalej
post_undo_repost = Cofnij podanie dalej
post_quote = Cytuj
post_reposts_label = Podania dalej i cytaty ({ $count })
post_like_label = Polub ({ $count })
post_bookmark_label = Dodaj do zakładek
post_share = Udostępnij lub skopiuj link
cw_show = Pokaż
cw_hide = Ukryj
cw_label = Ostrzeżenie o treści: { $reason }
muted_toast = Wyciszono { $handle }
unmuted_toast = Wyłączono wyciszenie { $handle }
blocked_toast = Zablokowano { $handle }
unblocked_toast = Odblokowano { $handle }
block_title = Zablokować { $handle }?
block_body = Ta osoba nie będzie mogła Cię obserwować, a Ty nie zobaczysz jej wpisów. Nie zostanie o tym powiadomiona.
block_confirm = Zablokuj
notify_like = @{ $handle } polubił(a) Twój wpis
notify_repost = @{ $handle } podał(a) dalej Twój wpis
notify_reply = @{ $handle } odpowiedział(a) Ci
notify_mention = @{ $handle } wspomniał(a) o Tobie
notify_quote = @{ $handle } zacytował(a) Twój wpis
notify_follow = @{ $handle } zaczął(-ęła) Cię obserwować
notify_follow_request = @{ $handle } chce Cię obserwować
notify_poll = Ankieta, w której głosowano, dobiegła końca
notify_affiliation = @{ $handle } zaprasza Cię do afiliacji
shortcuts_title = Skróty klawiszowe
shortcut_group_navigation = Nawigacja
shortcut_group_actions = Akcje
shortcut_or = /
shortcut_then = potem
shortcut_plus = +
shortcut_next_prev = Następny / poprzedni wpis
shortcut_open = Otwórz wpis
shortcut_search = Szukaj
shortcut_go_home = Główna
shortcut_go_explore = Odkrywaj
shortcut_go_notifications = Powiadomienia
shortcut_go_bookmarks = Zakładki
shortcut_go_profile = Twój profil
shortcut_go_settings = Ustawienia
shortcut_new_post = Nowy wpis
shortcut_like = Polub
shortcut_reply = Odpowiedz
shortcut_repost = Podaj dalej lub cytuj
shortcut_bookmark = Dodaj do zakładek
shortcut_send = Wyślij wpis
shortcut_help = Pokaż tę pomoc
auth_login_title = Zaloguj się
auth_add_title = Dodaj kolejne konto
auth_add_sub = Zaloguj się, aby przełączać się między maksymalnie { $max } kontami na tym urządzeniu bez wylogowywania.
auth_signup_title = Utwórz konto
auth_handle = Nazwa użytkownika
auth_password = Hasło
auth_display_name = Wyświetlana nazwa
auth_handle_hint = 3–15 liter, cyfr lub podkreśleń
auth_password_hint = Co najmniej 8 znaków
auth_consent_pre = Mam co najmniej { $minAge } lat i akceptuję{" "}
auth_consent_terms = Regulamin
auth_consent_and = {" "}oraz{" "}
auth_consent_privacy = Polityka prywatności
auth_consent_end = {"."}
auth_consent_required = Zaakceptuj Regulamin i Politykę prywatności.
auth_signin = Zaloguj się
auth_signin_add = Zaloguj się i przełącz
auth_create = Utwórz konto
auth_to_signup = Pierwszy raz tutaj? Załóż konto
auth_to_login = Masz konto? Zaloguj się
auth_closed = Rejestracja na tej instancji jest teraz zamknięta.
landing_nav = Stopka
landing_rules = Zasady
landing_stats = Teraz
landing_people = Osoby
landing_posts = Wpisy
landing_clock = Twój czas lokalny
landing_trends = Popularne
landing_trend =
    { $count ->
        [one] { $count } wpis
        [few] { $count } wpisy
        [many] { $count } wpisów
       *[other] { $count } wpisu
    }
landing_join = Dołącz do { $name }
landing_join_sub = Załóż konto i publikuj w dwóch kolorach tuszu.
landing_closed = Rejestracja jest teraz zamknięta. Masz już konto? Zaloguj się.
landing_terms = Regulamin
landing_privacy = Prywatność
landing_source = Kod źródłowy
landing_powered = Działa na tuita { $version }
docs_terms_title = Regulamin
docs_privacy_title = Polityka prywatności
docs_updated = Ostatnia aktualizacja: { $date }
docs_date = 5 listopada 2026
docs_see_privacy = Zobacz też Politykę prywatności.
docs_see_terms = Zobacz też Regulamin.
docs_contact_fallback = administratorami instancji (wiadomością prywatną lub zgłoszeniem)
docs_terms_agreement_h = 1. Zgoda
docs_terms_agreement_p = Niniejszy Regulamin („Regulamin”) określa zasady korzystania z { $name }, instancji tuita pod adresem { $domain } („usługa”, „my”). Zakładając konto lub korzystając z usługi, akceptujesz ten Regulamin oraz naszą Politykę prywatności. Jeśli się nie zgadzasz, nie korzystaj z usługi.
docs_terms_eligibility_h = 2. Kto może korzystać
docs_terms_eligibility_p = Musisz mieć co najmniej { $minAge } lat albo wiek zgody cyfrowej obowiązujący tam, gdzie mieszkasz, jeśli jest wyższy. Musisz móc zawrzeć wiążącą umowę i nie możesz mieć zakazu korzystania z usługi na mocy obowiązujących przepisów.
docs_terms_account_h = 3. Twoje konto
docs_terms_account_1 = Zachowaj hasło w tajemnicy. Odpowiadasz za aktywność na swoim koncie.
docs_terms_account_2 = Twoja nazwa użytkownika nie może podszywać się pod inną osobę lub organizację ani wprowadzać innych w błąd co do tego, kim jesteś.
docs_terms_account_3 = Niezwłocznie powiadom nas, jeśli uważasz, że Twoje konto zostało przejęte. Pozostałe sesje możesz wylogować w Ustawienia → Konto.
docs_terms_content_h = 4. Twoje treści
docs_terms_content_p1 = Zachowujesz prawa do tego, co publikujesz. Udzielasz nam ogólnoświatowej, niewyłącznej, bezpłatnej licencji na przechowywanie, wyświetlanie, powielanie i rozpowszechnianie Twoich treści wyłącznie w celu działania i ulepszania usługi. Obejmuje to wysyłanie ich do połączonych instancji tuita (zob. punkt 6). Licencja wygasa, gdy usuniesz treść lub konto, z wyjątkiem kopii już dostarczonych do innych instancji lub zapisanych przez innych użytkowników.
docs_terms_content_p2 = Odpowiadasz za to, co publikujesz. Publikuj tylko treści, które masz prawo udostępniać.
docs_terms_rules_h = 5. Zasady społeczności
docs_terms_rules_lead = Korzystając z { $name }, zobowiązujesz się nie łamać prawa ani tych zasad:
docs_terms_rules_p = Zobowiązujesz się też nie atakować usługi, na przykład próbując obejść zabezpieczenia, masowo pobierać danych, wysyłać złośliwego oprogramowania ani jej przeciążać.
docs_terms_federation_h = 6. Federacja
docs_terms_federation_p = { $name } może łączyć się („federować”) z innymi niezależnymi instancjami tuita zatwierdzonymi przez administratorów. Gdy wchodzisz w interakcję z osobami z innej instancji, Twój publiczny profil, publiczne wpisy i sama interakcja (na przykład obserwowanie, polubienie, podanie dalej, odpowiedź lub wiadomość prywatna) są do niej wysyłane. Inne instancje prowadzą inne osoby na własnych zasadach. Gdy treść trafi do innej instancji, nie możemy jej kontrolować ani usunąć, choć wysyłamy tam prośby o usunięcie, gdy coś usuniesz.
docs_terms_moderation_h = 7. Moderacja i egzekwowanie zasad
docs_terms_moderation_p = Moderatorzy mogą usuwać treści, ograniczać funkcje lub zawieszać konta, które naruszają ten Regulamin lub prawo. Jeśli to możliwe, wyjaśnimy dlaczego. Każdy może zgłosić wpis lub konto. Zgłoszenia trafiają do moderatorów tej instancji. Możemy w dowolnym momencie przestać federować z dowolną instancją.
docs_terms_termination_h = 8. Rezygnacja
docs_terms_termination_p = W dowolnej chwili możesz wyeksportować swoje dane lub usunąć konto w Ustawienia → Konto. Możemy zawiesić lub zakończyć Twój dostęp, jeśli poważnie lub wielokrotnie naruszysz ten Regulamin. Możemy też zamknąć usługę i postaramy się wcześniej odpowiednio poinformować.
docs_terms_disclaimers_h = 9. Wyłączenia i odpowiedzialność
docs_terms_disclaimers_p = Usługa jest udostępniana w stanie „takim, jaki jest” i „w miarę dostępności”, bez gwarancji jakiegokolwiek rodzaju, w najszerszym zakresie dozwolonym przez prawo. Nie odpowiadamy za szkody pośrednie, uboczne ani następcze ani za treści publikowane przez użytkowników lub inne instancje. Nic w tym Regulaminie nie ogranicza odpowiedzialności, której nie można ograniczyć z mocy prawa, ani Twoich ustawowych praw konsumenta.
docs_terms_changes_h = 10. Zmiany
docs_terms_changes_p = Możemy aktualizować ten Regulamin. Jeśli zmiana będzie istotna, powiadomimy o niej w aplikacji przed jej wejściem w życie. Dalsze korzystanie z usługi po zmianie oznacza akceptację nowego Regulaminu.
docs_terms_contact_h = 11. Kontakt
docs_terms_contact_lead = Pytania dotyczące tego Regulaminu:
docs_privacy_summary_h = W skrócie
docs_privacy_summary_1 = Bez reklam, bez skryptów śledzących, bez sprzedaży danych.
docs_privacy_summary_2 = Jedno ciasteczko, aby utrzymać Cię w sesji. Preferencje motywu zostają w Twojej przeglądarce.
docs_privacy_summary_3 = Publiczne wpisy są publiczne i mogą trafić do połączonych instancji tuita.
docs_privacy_summary_4 = Wiadomości prywatne między kontami na tym serwerze są szyfrowane end-to-end, gdy obie osoby włączą szyfrowanie: są przechowywane jako tekst, którego serwer nie może odczytać, więc wyciek bazy danych ani wgląd w przechowywane dane niczego nie ujawnia. Wiadomości do osób z innych serwerów oraz wiadomości wysłane przed włączeniem szyfrowania nie są szyfrowane w ten sposób i administratorzy technicznie mogą je odczytać. Ten serwer dostarcza też aplikację, która szyfruje, więc administrator, który zmieniłby tę aplikację, mógłby przechwycić to, co wpisujesz.
docs_privacy_summary_5 = Możesz w każdej chwili wyeksportować lub usunąć swoje dane.
docs_privacy_controller_h = 1. Kim jesteśmy
docs_privacy_controller_p = Ta polityka dotyczy { $name } pod adresem { $domain }, prowadzonej przez administratorów instancji. To oni decydują, jak przetwarzane są Twoje dane.
docs_privacy_collect_h = 2. Co zbieramy
docs_privacy_collect_1 = Dane konta: nazwa użytkownika, wyświetlana nazwa, hasło (przechowujemy tylko jego skrót argon2id z solą) oraz dane profilu: opis, lokalizacja, strona internetowa, awatar, baner, kształt i kolor.
docs_privacy_collect_2 = Treści: wpisy, obrazy, odpowiedzi, polubienia, podania dalej, zakładki, obserwowania, blokady, wyciszenia i zgłoszenia.
docs_privacy_collect_3 = Wiadomości prywatne: przechowywane na tym serwerze, aby można je było dostarczyć Tobie i drugiej osobie. Wiadomość szyfrowana end-to-end jest przechowywana jako szyfrogram, którego nie możemy odczytać; kto ją wysłał, do kogo i kiedy, pozostaje dla nas widoczne. Pozostałe wiadomości są przechowywane jako zwykły tekst. Twój klucz szyfrowania jest tu przechowywany tylko w postaci zablokowanej Twoją frazą hasłową, której nigdy nie widzimy.
docs_privacy_collect_4 = Dane sesji: dla każdego zalogowanego urządzenia user-agent przeglądarki, adres IP i czas ostatniego użycia, abyś mógł(-ła) przeglądać sesje i się z nich wylogowywać.
docs_privacy_collect_5 = Krótkotrwałe dane bezpieczeństwa: adresy IP przechowywane w pamięci przez kilka minut w celu ograniczania liczby żądań.
docs_privacy_collect_p = Nie zbieramy Twojego adresu e-mail, numeru telefonu, kontaktów ani lokalizacji, chyba że sam(a) wpiszesz je w profilu.
docs_privacy_use_h = 3. Jak ich używamy
docs_privacy_use_p = Tylko do działania usługi: pokazywania Twojego profilu i wpisów, budowania osi czasu i powiadomień, dostarczania wiadomości, zabezpieczania kont, zapobiegania nadużyciom i egzekwowania Regulaminu. Oś czasu „Dla Ciebie” układa wpisy według polubień, podań dalej, odpowiedzi i wieku. Nie profiluje Cię.
docs_privacy_visibility_h = 4. Co widzą inni
docs_privacy_visibility_p = Twój profil, wpisy, odpowiedzi, podania dalej, polubienia, obserwujący i obserwowani są publiczne dla każdego zalogowanego na tej instancji oraz dla połączonych instancji. Gdy ktoś udostępni link do wpisu lub profilu, aplikacje takie jak Discord, Slack czy iMessage mogą pobrać podgląd z publicznym wpisem, jego obrazami i publicznymi danymi Twojego profilu. Wpisy z ostrzeżeniem o treści lub oznaczone jako wrażliwe mają podgląd bez tekstu i obrazów. Zakładki, blokady, wyciszenia, zgłoszenia i ustawienia są prywatne. Jeśli wyłączysz opcję „Pokazuj mnie w propozycjach”, inni nadal mogą Cię znaleźć po dokładnej nazwie użytkownika.
docs_privacy_federation_h = 5. Federacja
docs_privacy_federation_p = Gdy obserwujesz osobę z innej instancji tuita, odpowiadasz jej, wspominasz o niej, lubisz lub podajesz dalej jej wpis albo piszesz do niej, wysyłamy do tej instancji potrzebne dane: Twój publiczny profil, wpis lub interakcję oraz treść wiadomości w przypadku wiadomości prywatnych. Wiadomości prywatne do innych instancji nie są szyfrowane end-to-end. Instancje Twoich obserwujących otrzymują Twoje publiczne wpisy i aktualizacje profilu. Te instancje działają niezależnie i przechowują dane zgodnie z własnymi politykami. Gdy usuniesz wpis lub konto, prosimy połączone instancje o usunięcie kopii, ale nie możemy zagwarantować, że to zrobią.
docs_privacy_cookies_h = 6. Ciasteczka i pamięć lokalna
docs_privacy_cookies_p = Używamy jednego bezwzględnie koniecznego ciasteczka sesji HttpOnly. Preferencje motywu, koloru, rozmiaru tekstu i animacji są zapisywane w pamięci lokalnej przeglądarki i nigdy do nas nie trafiają.
docs_privacy_retention_h = 7. Jak długo przechowujemy dane
docs_privacy_retention_1 = Treści i dane konta są przechowywane, dopóki ich nie usuniesz.
docs_privacy_retention_2 = Usunięte wpisy natychmiast znikają z widoku publicznego, a ich tekst i obrazy są czyszczone.
docs_privacy_retention_3 = Gdy usuniesz konto, usuwamy Twoje wpisy, polubienia, obserwowania, wiadomości i sesje oraz anonimizujemy rekord konta.
docs_privacy_retention_4 = Sesje wygasają po 30 dniach lub od razu po wylogowaniu.
docs_privacy_retention_5 = Zgłoszenia moderacyjne i dziennik audytu administracji są przechowywane tak długo, jak to potrzebne dla bezpieczeństwa usługi.
docs_privacy_rights_h = 8. Twoje prawa
docs_privacy_rights_p = W zależności od miejsca zamieszkania (na przykład na mocy RODO lub CCPA) możesz mieć prawo dostępu do danych, ich poprawiania, eksportu lub usunięcia oraz sprzeciwu wobec przetwarzania lub jego ograniczenia. Większość z tego zrobisz sam(a) w Ustawienia → Konto: edytuj profil, pobierz dane i usuń konto. W pozostałych sprawach skontaktuj się z nami:
docs_privacy_rights_end = Możesz też złożyć skargę do lokalnego organu ochrony danych.
docs_privacy_security_h = 9. Bezpieczeństwo
docs_privacy_security_p = Hasła są haszowane algorytmem scrypt. Tokeny sesji przechowujemy tylko w postaci skrótów. Żądania między połączonymi instancjami są podpisywane kluczami Ed25519. Żaden system nie jest w pełni bezpieczny, więc używaj unikalnego hasła.
docs_privacy_children_h = 10. Dzieci
docs_privacy_children_p = Usługa nie jest przeznaczona dla dzieci poniżej minimalnego wieku podanego w Regulaminie. Jeśli uważasz, że dziecko ma konto, zgłoś to.
docs_privacy_changes_h = 11. Zmiany
docs_privacy_changes_p = Zmiany tej polityki opublikujemy tutaj i zaktualizujemy datę powyżej. Istotne zmiany ogłosimy w aplikacji.
docs_privacy_contact_h = 12. Kontakt
docs_privacy_contact_lead = Pytania i wnioski dotyczące prywatności:
profile_posts =
    { $count ->
        [one] { $compact } wpis
        [few] { $compact } wpisy
        [many] { $compact } wpisów
       *[other] { $compact } wpisu
    }
profile_edit = Edytuj profil
profile_more = Więcej
profile_copy_link = Kopiuj link do profilu
profile_mute = Wycisz { $handle }
profile_unmute = Wyłącz wyciszenie { $handle }
profile_block = Zablokuj { $handle }
profile_unblock = Odblokuj { $handle }
profile_unblock_button = Odblokuj
profile_suspended = To konto jest zawieszone. Widzi je tylko administracja.
profile_muted = Wyciszono
profile_org = Organizacja
profile_role_admin = Administracja
profile_role_moderator = Moderator
profile_joined = Dołączył(a) { $date }
profile_following_label = Obserwowani
profile_followers =
    { $count ->
        [one] Obserwujący
        [few] Obserwujących
        [many] Obserwujących
       *[other] Obserwujących
    }
profile_counts = Liczba obserwowanych i obserwujących
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Obserwuje: { $a }
            }
        [2]
            { $others ->
               *[other] Obserwują: { $a } i { $b }
            }
       *[other]
            { $others ->
                [one] Obserwują: { $a }, { $b } i { $others } inna osoba, którą obserwujesz
                [few] Obserwują: { $a }, { $b } i { $others } inne osoby, które obserwujesz
                [many] Obserwują: { $a }, { $b } i { $others } innych osób, które obserwujesz
               *[other] Obserwują: { $a }, { $b } i { $others } innej osoby, którą obserwujesz
            }
    }
profile_banner = Baner profilu
profile_tabs = Sekcje profilu
profile_tab_posts = Wpisy
profile_tab_replies = Odpowiedzi
profile_tab_media = Multimedia
profile_tab_likes = Polubienia
profile_posts_label = Wpisy
profile_empty_posts_self_title = Nie masz jeszcze wpisów
profile_empty_posts_self_body = Tu pojawią się Twoje wpisy i podania dalej.
profile_empty_posts_title = { $handle } nic jeszcze nie opublikował(a)
profile_empty_posts_body = Gdy to zrobi, wpisy pojawią się tutaj.
profile_empty_replies_title = Brak odpowiedzi
profile_empty_replies_body = Tu pojawią się odpowiedzi.
profile_empty_media_title = Brak zdjęć
profile_empty_media_body = Tu pojawią się wpisy z obrazami.
profile_empty_likes_title = Brak polubień
profile_empty_likes_body = Tu pojawią się polubione wpisy.
profile_blocked_title = Zablokowano { $handle }
profile_blocked_body = Nie zobaczysz wpisów tej osoby, a ona nie może Cię obserwować ani do Ciebie pisać.
profile_hidden_title = Nie możesz zobaczyć tych wpisów
profile_hidden_body = { $handle } ograniczył(a), kto może zobaczyć profil.
follows_tabs = Listy obserwowania
follows_tab_followers = Obserwujący
follows_tab_following = Obserwowani
follows_label_followers = Obserwujący
follows_label_following = Obserwowani
follows_empty_followers_title = Brak obserwujących
follows_empty_followers_body = Gdy ktoś zacznie obserwować to konto, pojawi się tutaj.
follows_empty_following_title = Nikogo nie obserwuje
follows_empty_following_body = Obserwowane konta pojawią się tutaj.
thread_title = Wpis
thread_doc_title = { $name }: „{ $text }”
thread_views =
    { $count ->
        [one] Wyświetlenie
        [few] Wyświetlenia
        [many] Wyświetleń
       *[other] Wyświetlenia
    }
thread_reposts =
    { $count ->
        [one] Podanie dalej
        [few] Podania dalej
        [many] Podań dalej
       *[other] Podania dalej
    }
thread_quotes =
    { $count ->
        [one] Cytat
        [few] Cytaty
        [many] Cytatów
       *[other] Cytatu
    }
thread_likes =
    { $count ->
        [one] Polubienie
        [few] Polubienia
        [many] Polubień
       *[other] Polubienia
    }
thread_stats = Statystyki wpisu
thread_replies = Odpowiedzi
thread_hidden = Ukryte odpowiedzi ({ $count })
thread_hidden_note = Autor ukrył te odpowiedzi.
thread_list_likes = Polubione przez
thread_list_reposts = Podane dalej przez
thread_list_quotes = Cytaty
thread_list_empty = Jeszcze nikt
tag_empty_title = Brak wpisów z { $tag }
tag_empty_body = Bądź pierwszy(-a), kto go użyje.
tag_posts_label = Wpisy z tagiem { $tag }
bookmarks_title = Zakładki
bookmarks_label = Wpisy w zakładkach
bookmarks_empty_title = Zapisuj wpisy na później
bookmarks_empty_body = Dotknij ikony zakładki przy wpisie, aby go tu zachować. Tylko Ty widzisz swoje zakładki.
settings_account = Twoje konto
settings_account_desc = Profil i hasło
settings_profile = Edytuj profil
settings_profile_desc = Nazwa, opis, kształt awatara i kolor tuszu
settings_privacy = Prywatność i bezpieczeństwo
settings_privacy_desc = Wiadomości, widoczność, wyciszone słowa, zablokowane i wyciszone konta
settings_notifications = Powiadomienia
settings_notifications_desc = Wybierz, o czym chcesz dostawać powiadomienia
settings_appearance = Wygląd
settings_appearance_desc = Motyw, kolor tuszu, rozmiar tekstu, kontrast i animacje
settings_language = Język
settings_language_desc = Język interfejsu
settings_language_note = Zapisywany w tej przeglądarce, a po zalogowaniu także na Twoim koncie.
settings_about = O tej instancji
settings_about_desc = Zasady, regulamin, prywatność, wersja i skróty klawiszowe
settings_logout = Wyloguj się
settings_account_profile = Profil
settings_account_joined = Dołączył(a) { $date }
settings_account_edit = Edytuj profil
settings_password_title = Zmień hasło
settings_password_current = Obecne hasło
settings_password_new = Nowe hasło
settings_password_hint = Co najmniej 8 znaków. Inne urządzenia zostaną wylogowane.
settings_password_confirm = Potwierdź nowe hasło
settings_password_submit = Zaktualizuj hasło
settings_password_mismatch = Nowe hasła nie są takie same.
settings_password_updated = Zaktualizowano hasło. Inne sesje zostały wylogowane.
settings_sessions_title = Urządzenia i sesje
settings_sessions_desc = Wyloguj wszystko, czego nie rozpoznajesz, a potem zmień hasło.
settings_device_on = { $browser } na { $os }
settings_device_unknown = Nieznane urządzenie
settings_session_current = To urządzenie
settings_session_last_active = Ostatnio aktywna: { $time }
settings_session_sign_out = Wyloguj
settings_session_sign_out_device = Wyloguj { $device }
settings_session_signed_out = Wylogowano sesję
settings_sessions_revoke_all = Wyloguj ze wszystkich innych urządzeń
settings_sessions_revoke_title = Wylogować inne urządzenia?
settings_sessions_revoke_body = Wszystkie urządzenia poza tym zostaną wylogowane.
settings_sessions_revoke_confirm = Wyloguj pozostałe
settings_sessions_revoked =
    { $count ->
        [one] Wylogowano { $count } sesję
        [few] Wylogowano { $count } sesje
        [many] Wylogowano { $count } sesji
       *[other] Wylogowano { $count } sesji
    }
settings_profile_preview = Podgląd
settings_profile_preview_alt = Podgląd awatara
settings_profile_uploads = Przesyłanie awatara i banera pojawi się później. Do tego czasu używane są wygenerowane awatar i baner.
settings_profile_details = Szczegóły
settings_profile_name = Wyświetlana nazwa
settings_profile_bio = Opis
settings_profile_location = Lokalizacja
settings_profile_website = Strona internetowa
settings_profile_shape = Kształt awatara
settings_profile_ink = Kolor tuszu profilu
settings_profile_save = Zapisz profil
settings_profile_saved = Zaktualizowano profil
settings_shape_square = Kwadrat
settings_shape_circle = Koło
settings_shape_squircle = Zaokrąglony kwadrat
settings_shape_octagon = Ośmiokąt
settings_ink_bone = Kość słoniowa
settings_ink_phosphor = Fosfor
settings_ink_amber = Bursztyn
settings_ink_cyan = Cyjan
settings_ink_rose = Róża
settings_dm_title = Wiadomości prywatne
settings_dm_desc = Kto może rozpocząć z Tobą rozmowę
settings_dm_label = Kto może do Ciebie pisać
settings_dm_everyone = Wszyscy
settings_dm_following = Osoby, które obserwuję
settings_dm_none = Nikt
settings_dm_saved = Zapisano ustawienia wiadomości
settings_discover_title = Widoczność
settings_discoverable = Pokazuj mnie w propozycjach i wyszukiwaniu
settings_discoverable_hint = Nadal można Cię znaleźć po dokładnej nazwie użytkownika
settings_show_client = Pokazuj, z jakiego urządzenia publikuję
settings_show_client_hint = Dodaje urządzenie pod Twoimi nowymi wpisami
settings_words_title = Wyciszone słowa
settings_words_desc = Ukrywaj wpisy z tymi słowami, frazami lub #hashtagami na osiach czasu, w wyszukiwaniu i odpowiedziach. Twoje własne wpisy nigdy nie są ukrywane.
settings_words_label = Słowo do wyciszenia
settings_words_placeholder = Dodaj słowo, frazę lub #hashtag
settings_words_add = Wycisz
settings_words_remove = Wyłącz wyciszenie: { $word }
settings_words_none = Brak wyciszonych słów.
settings_words_limit = Limit wyciszonych słów: { $count }.
settings_blocked_title = Zablokowane konta
settings_blocked_desc = Te osoby nie mogą Cię obserwować, pisać do Ciebie ani widzieć Twoich wpisów i wchodzić z nimi w interakcje. Nie są o tym powiadamiane.
settings_blocked_empty = Nikogo nie zablokowano.
settings_muted_title = Wyciszone konta
settings_muted_desc = Nie zobaczysz ich wpisów ani powiadomień. Nadal mogą Cię obserwować i do Ciebie pisać.
settings_muted_empty = Nikogo nie wyciszono.
settings_unblock = Odblokuj
settings_unmute = Wyłącz wyciszenie
settings_unblock_user = Odblokuj { $handle }
settings_unmute_user = Wyłącz wyciszenie { $handle }
settings_notif_title = Powiadamiaj mnie o
settings_notif_desc = Wyciszone i zablokowane konta nigdy nie wysyłają Ci powiadomień.
settings_notif_like = Polubienia
settings_notif_like_hint = Gdy ktoś polubi Twój wpis
settings_notif_repost = Podania dalej
settings_notif_repost_hint = Gdy ktoś poda dalej Twój wpis
settings_notif_reply = Odpowiedzi
settings_notif_reply_hint = Gdy ktoś Ci odpowie
settings_notif_mention = Wzmianki
settings_notif_mention_hint = Gdy ktoś o Tobie wspomni
settings_notif_quote = Cytaty
settings_notif_quote_hint = Gdy ktoś zacytuje Twój wpis
settings_notif_follow = Nowi obserwujący
settings_notif_follow_hint = Gdy ktoś zacznie Cię obserwować
settings_desktop_title = Powiadomienia systemowe
settings_desktop_desc = Wyświetlane jako powiadomienie systemowe tylko wtedy, gdy ta karta jest w tle.
settings_desktop_state = Uprawnienie: { $state }
settings_desktop_allowed = Dozwolone
settings_desktop_blocked = Zablokowane
settings_desktop_not_asked = Nie zapytano
settings_desktop_enable = Włącz
settings_desktop_blocked_toast = Powiadomienia dla tej witryny są zablokowane w ustawieniach przeglądarki.
settings_desktop_unsupported = Ta przeglądarka nie obsługuje powiadomień systemowych.
settings_theme_title = Motyw
settings_appearance_note = Zapisywane na tym urządzeniu i stosowane od razu.
settings_theme_mode = Motyw
settings_theme_system = Systemowy
settings_theme_light = Jasny
settings_theme_dark = Ciemny
settings_theme_ink = Kolor tuszu
settings_theme_contrast = Kontrast
settings_contrast_normal = Normalny
settings_contrast_more = Więcej
settings_reading_title = Tekst i animacje
settings_theme_text_size = Rozmiar tekstu
settings_theme_motion = Animacje
settings_motion_system = Systemowy
settings_motion_reduce = Ogranicz
settings_preview_title = Podgląd
settings_preview_text = Tak będą wyglądać wpisy przy Twoich ustawieniach.
settings_about_domain = Domena
settings_about_version = Wersja
settings_about_contact = Kontakt
settings_about_terms = Regulamin
settings_about_privacy = Polityka prywatności
settings_about_source = Kod źródłowy
settings_about_rules = Zasady
explore_search_label = Szukaj wpisów, osób i tagów
explore_search_placeholder = Szukaj wpisów, osób, #tagów
explore_clear = Wyczyść wyszukiwanie
explore_tips = Wskazówki wyszukiwania
explore_tips_title = Operatory wyszukiwania
explore_op_from = Wpisy danego konta
explore_op_to = Odpowiedzi lub wzmianki dla danego konta
explore_op_media = Wpisy z obrazami (lub has:poll, has:links)
explore_op_since = Opublikowane w dniu lub po dacie
explore_op_until = Opublikowane w dniu lub przed datą
explore_op_phrase = Słowa w tej kolejności
explore_op_exclude = Pomiń wpisy z danym słowem
explore_results = Wyniki wyszukiwania
explore_tab_posts = Wpisy
explore_tab_people = Osoby
explore_tab_media = Multimedia
explore_people = Osoby
explore_posts = Wpisy
explore_recent = Ostatnie wyszukiwania
explore_recent_clear = Wyczyść
explore_trending = Popularne
explore_trend_posts =
    { $count ->
        [one] { $compact } wpis
        [few] { $compact } wpisy
        [many] { $compact } wpisów
       *[other] { $compact } wpisu
    }
explore_trends_empty = Brak trendów. Użyj #hashtagu!
explore_suggestions = Kogo obserwować
explore_suggestions_empty = Obserwujesz wszystkich. Imponujące.
explore_no_posts_title = Brak wpisów dla „{ $q }”
explore_no_posts_body = Spróbuj innego słowa lub hashtagu.
explore_no_people_title = Brak osób dla „{ $q }”
explore_no_people_body = Wpisz imię, nazwę użytkownika lub @nazwa@instancja, aby znaleźć kogoś z innej instancji.
explore_no_media_title = Brak obrazów dla „{ $q }”
explore_no_media_body = Tu pojawią się pasujące wpisy z obrazami.
notif_filter = Filtr powiadomień
notif_tab_all = Wszystkie
notif_tab_mentions = Wzmianki
notif_empty_all_title = Brak powiadomień
notif_empty_all_body = Tu pojawią się polubienia, odpowiedzi, podania dalej i nowi obserwujący.
notif_empty_mentions_title = Brak wzmianek
notif_empty_mentions_body = Tu pojawią się odpowiedzi, wzmianki i cytaty Twoich wpisów.
notif_who_two = { $first } i { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } i { $count } inna osoba
        [few] { $first }, { $second } i { $count } inne osoby
        [many] { $first }, { $second } i { $count } innych osób
       *[other] { $first }, { $second } i { $count } innej osoby
    }
notif_verb_like =
    { $count ->
        [one] polubił(a) Twój wpis
        [few] polubili Twój wpis
        [many] polubili Twój wpis
       *[other] polubili Twój wpis
    }
notif_verb_repost =
    { $count ->
        [one] podał(a) dalej Twój wpis
        [few] podali dalej Twój wpis
        [many] podali dalej Twój wpis
       *[other] podali dalej Twój wpis
    }
notif_verb_follow =
    { $count ->
        [one] zaczął(-ęła) Cię obserwować
        [few] zaczęli Cię obserwować
        [many] zaczęli Cię obserwować
       *[other] zaczęli Cię obserwować
    }
notif_verb_follow_request =
    { $count ->
        [one] chce Cię obserwować
        [few] chcą Cię obserwować
        [many] chcą Cię obserwować
       *[other] chcą Cię obserwować
    }
notif_verb_reply =
    { $count ->
        [one] odpowiedział(a) Ci
        [few] odpowiedzieli Ci
        [many] odpowiedzieli Ci
       *[other] odpowiedzieli Ci
    }
notif_verb_mention =
    { $count ->
        [one] wspomniał(a) o Tobie
        [few] wspomnieli o Tobie
        [many] wspomnieli o Tobie
       *[other] wspomnieli o Tobie
    }
notif_verb_quote =
    { $count ->
        [one] zacytował(a) Twój wpis
        [few] zacytowali Twój wpis
        [many] zacytowali Twój wpis
       *[other] zacytowali Twój wpis
    }
notif_verb_poll =
    { $count ->
        [one] zakończył(a) ankietę, w której głosowano
        [few] zakończyli ankietę, w której głosowano
        [many] zakończyli ankietę, w której głosowano
       *[other] zakończyli ankietę, w której głosowano
    }
notif_verb_affiliation =
    { $count ->
        [one] zaprasza Cię do afiliacji
        [few] zapraszają Cię do afiliacji
        [many] zapraszają Cię do afiliacji
       *[other] zapraszają Cię do afiliacji
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Zaplanuj
composer_post_all = Opublikuj wszystko
composer_err_type = Ten plik nie jest obrazem ani filmem obsługiwanym przez tuita (PNG, JPEG, GIF, WebP, MP4, WebM lub MOV).
composer_err_image_size = Obraz jest za duży (maks. 10 MB).
composer_err_video_size = Filmy mogą mieć do { $mb } MB.
composer_err_max_images = Możesz dołączyć maksymalnie 4 obrazy.
composer_err_exclusive = Wpis może mieć film, obrazy albo ankietę, ale tylko jedno z nich.
composer_extra_placeholder = Dodaj kolejny wpis
composer_extra_label = Wpis { $n } w wątku
composer_extra_remove = Usuń wpis { $n } w wątku
composer_schedule_chip = Opublikuje się { $time }
composer_schedule_clear = Nie planuj
composer_add_images = Dodaj obrazy
composer_add_video = Dodaj film
composer_add_poll = Dodaj ankietę
composer_add_post = Dodaj kolejny wpis
composer_scheduled = Zaplanowano na { $time }
composer_thread_sent = Wysłano wątek z liczbą wpisów: { $count }
composer_remove_video = Usuń film
composer_remove_image = Usuń obraz
composer_uploading = Przesyłanie
composer_uploading_video = Przesyłanie filmu… { $percent }%
composer_alt_video = Opisz ten film
composer_alt_image = Opisz ten obraz
composer_alt_label = Opis (tekst alternatywny) załącznika { $index }
composer_emoji = Dodaj emoji
poll_length_hours = { $n } godz.
poll_length_days = { $n } d
poll_choice = Opcja { $n }
poll_choice_optional = Opcja { $n } (opcjonalnie)
poll_remove_choice = Usuń opcję { $n }
poll_add_choice = Dodaj opcję
poll_length = Czas trwania ankiety
poll_multiple_label = Zezwól na wiele odpowiedzi
poll_remove = Usuń ankietę
schedule_title = Zaplanuj wpis
schedule_hour = Za godzinę
schedule_evening = Dziś wieczorem
schedule_tomorrow = Jutro rano
schedule_week = Za tydzień
schedule_window = Wybierz czas od jednej minuty do 90 dni od teraz.
schedule_custom = Wybierz datę i godzinę
schedule_set = Ustaw
schedule_clear = Nie planuj
schedule_view = Zobacz zaplanowane wpisy
dropzone_drop = Upuść, aby dołączyć
dropzone_reject = Nie można użyć tego pliku
nav_messages = Wiadomości
nav_lists = Listy
nav_scheduled = Zaplanowane
notify_message = @{ $handle } wysłał(a) Ci wiadomość
shortcut_go_messages = Wiadomości
shortcut_go_lists = Listy
cropper_stage = Obszar kadrowania. Przeciągnij, aby przesunąć obraz. Strzałkami przesuniesz go delikatnie, a plusem i minusem przybliżysz lub oddalisz.
cropper_hint = Przeciągnij, aby przesunąć. Uszczypnij, przewiń lub użyj suwaka, aby powiększyć.
cropper_zoom = Powiększenie
cropper_zoom_in = Powiększ
cropper_zoom_out = Pomniejsz
cropper_rotate = Obróć
cropper_reset = Resetuj
cropper_apply = Zastosuj
image_field_avatar = Zdjęcie profilowe
image_field_banner = Baner
image_field_edit_avatar = Edytuj zdjęcie profilowe
image_field_edit_banner = Edytuj baner
image_field_change = Zmień
image_field_remove = Usuń
image_field_hint = PNG, JPEG, GIF lub WebP do 10 MB. GIF zostanie zamieniony na nieruchomy obraz.
image_field_not_image = Ten plik nie jest obrazem obsługiwanym przez tuita (PNG, JPEG, GIF lub WebP).
image_field_too_large = Obraz jest za duży (maks. 10 MB).
image_field_uploading = Przesyłanie
image_field_saved = Zapisano obraz.
image_field_removed = Usunięto obraz.
edit_title = Edytuj wpis
edit_note = Wpis możesz edytować przez 30 minut od publikacji. Inni widzą, że wpis edytowano, oraz jego wcześniejsze wersje.
edit_save = Zapisz
edit_saved = Zaktualizowano wpis
edit_edited = Edytowano
edit_history_title = Historia edycji
edit_history_current = Bieżąca wersja
edit_history_earlier = Wcześniejsza wersja
edit_history_original = Oryginał
report_post_title = Zgłoś wpis
report_user_title = Zgłoś { $handle }
report_intro = Co jest nie tak? Moderatorzy tej instancji sprawdzą Twoje zgłoszenie. { $handle } nie zobaczy, kto je wysłał.
report_reason = Powód
report_note = Coś jeszcze? (opcjonalnie)
report_send = Wyślij zgłoszenie
report_sent = Dziękujemy. Zgłoszenie trafiło do moderatorów.
report_duplicate = Już to zgłoszono. Moderatorzy się temu przyjrzą.
report_spam = Spam
report_spam_body = Niechciane reklamy, oszustwa, powtarzające się lub wprowadzające w błąd treści
report_harassment = Nękanie
report_harassment_body = Atakowanie, zastraszanie lub grożenie komuś
report_hate = Nienawiść
report_hate_body = Atakuje ludzi za to, kim są
report_violence = Przemoc lub samookaleczenie
report_violence_body = Groźby, gloryfikowanie przemocy, zachęcanie do samookaleczeń
report_misinformation = Dezinformacja
report_misinformation_body = Fałszywe twierdzenia, które mogą zaszkodzić
report_impersonation = Podszywanie się
report_impersonation_body = Udawanie kogoś innego
report_other = Coś innego
report_other_body = Łamie zasady w inny sposób
post_edit = Edytuj wpis
post_edit_history = Historia edycji
post_report = Zgłoś wpis
post_hide_reply = Ukryj odpowiedź
post_unhide_reply = Pokaż odpowiedź
post_share_menu = Udostępnij…
post_reply_hidden = Ukryta odpowiedź
post_reply_hidden_toast = Ukryto odpowiedź
post_reply_unhidden_toast = Odpowiedź znów widoczna
sensitive_label = Wrażliwe multimedia
sensitive_show = Pokaż
sensitive_hide = Ukryj wrażliwe multimedia
media_label =
    { $count ->
        [one] { $count } załącznik
        [few] { $count } załączniki
        [many] { $count } załączników
       *[other] { $count } załącznika
    }
media_video = Film
media_open = Otwórz obraz { $index } z { $count }
lightbox_title = Przeglądarka multimediów
lightbox_prev = Poprzedni
lightbox_next = Następny
poll_vote = Głosuj
poll_your_vote = Twój głos
poll_votes =
    { $count ->
        [one] { $compact } głos
        [few] { $compact } głosy
        [many] { $compact } głosów
       *[other] { $compact } głosu
    }
poll_final = Wyniki końcowe
poll_multiple = wielokrotny wybór
poll_left_minutes = Zostało { $n } min
poll_left_hours = Zostało { $n } godz.
poll_left_days = Zostało { $n } d
pull_refresh = Pociągnij, aby odświeżyć
messages_title = Wiadomości
messages_new = Nowa wiadomość
messages_label = Rozmowy
messages_empty_title = Rozpocznij rozmowę
messages_empty_body = Wysłane i otrzymane wiadomości pojawią się tutaj.
messages_you_prefix = Ty: { $text }
messages_unread = Nieprzeczytane: { $count }
messages_new_search = Szukaj osób
messages_new_results = Osoby
messages_new_none = Nikogo nie znaleziono.
chat_load_older = Wczytaj wcześniejsze wiadomości
chat_intro = To początek Waszej rozmowy.
chat_label = Wiadomości z { $name }
chat_delete = Usuń
chat_seen = Wyświetlono
chat_delete_title = Usunąć tę wiadomość?
chat_delete_body = Zniknie dla Was obojga.
chat_doc_title = Czat z { $name }
chat_error_title = Nie można otworzyć tej rozmowy
chat_placeholder = Napisz wiadomość
chat_input_label = Wiadomość
chat_send = Wyślij
lists_title = Listy
lists_new = Nowa lista
lists_edit = Edytuj listę
lists_label = Twoje listy
lists_empty_title = Brak list
lists_empty_body = Pogrupuj osoby na listach, aby czytać ich wpisy osobno.
lists_members =
    { $count ->
        [one] { $compact } członek
        [few] { $compact } członków
        [many] { $compact } członków
       *[other] { $compact } członka
    }
lists_private = Prywatna
lists_name = Nazwa
lists_description = Opis
lists_private_switch = Lista prywatna
lists_save = Zapisz
lists_picker_title = Dodaj { $handle } do list
lists_picker_none = Nie masz jeszcze list.
lists_delete = Usuń listę
lists_delete_title = Usunąć tę listę?
lists_delete_body = Lista „{ $name }” zostanie usunięta. Osoby na niej nie zostaną naruszone.
lists_by = Lista użytkownika { $handle }
lists_tab_posts = Wpisy
lists_tab_members = Członkowie
lists_tabs = Sekcje listy
lists_error_title = Nie można otworzyć tej listy
lists_posts_label = Wpisy z listy
lists_posts_empty_title = Jeszcze nic tu nie ma
lists_posts_empty_body = Wpisy członków tej listy pojawią się tutaj.
lists_members_label = Członkowie listy
lists_members_empty_title = Brak członków
lists_members_empty_body = Dodawaj osoby z ich profili.
lists_remove = Usuń
scheduled_title = Zaplanowane wpisy
scheduled_label = Zaplanowane wpisy
scheduled_status_pending = Zaplanowane
scheduled_status_published = Opublikowano
scheduled_status_failed = Nie powiodło się
scheduled_cw = Ostrzeżenie o treści: { $text }
scheduled_poll = Ankieta z liczbą opcji: { $count }, trwa { $hours } godz.
scheduled_change_time = Zmień godzinę
scheduled_post_now = Opublikuj teraz
scheduled_remove = Usuń
scheduled_window_error = Wybierz czas od jednej minuty do 90 dni od teraz.
scheduled_when = Opublikuj
scheduled_zone = Strefa czasowa: { $zone }
scheduled_save = Zapisz
scheduled_posted = Opublikowano.
scheduled_cancel_title = Anulować ten zaplanowany wpis?
scheduled_cancel_body = Nie zostanie opublikowany.
scheduled_cancel_confirm = Anuluj wpis
scheduled_empty_title = Nic nie zaplanowano
scheduled_empty_body = Zaplanowane wpisy czekają tutaj na swój czas.
profile_message = Wiadomość
profile_add_to_lists = Dodaj do list
media_alt = ALT
profile_report = Zgłoś { $handle }
chart_show_table = Pokaż { $title } jako tabelę
chart_col_day = Dzień
chart_col_hour = Godzina (UTC)
chart_empty = { $title }: na razie nic do pokazania.
chart_bar_summary = { $title }: łącznie { $total }, najwięcej { $label } ({ $peak }).
chart_spark_summary = { $title }: łącznie { $total }, { $last } dnia { $label }.
chart_heat_summary = { $title }: łącznie { $total }, największy ruch { $label } o { $hour } ({ $peak }).
chart_heat_less = Mniej
chart_heat_more = Więcej
announce_region = Ogłoszenia
announce_dismiss = Zamknij ogłoszenie
announce_more = Czytaj więcej
announce_tone_info = Informacja
announce_tone_success = Dobra wiadomość
announce_tone_warning = Ostrzeżenie
admin_stat_people = Osoby
admin_stat_people_sub = Aktywni w tym tygodniu: { $count }
admin_stat_posts_today = Wpisy, 24 godz.
admin_stat_posts_total = { $count } łącznie
admin_stat_reports = Otwarte zgłoszenia
admin_stat_reports_open = Wymaga przeglądu
admin_stat_reports_clear = Wszystko w porządku
admin_stat_signups_today = Rejestracje dziś
admin_stat_signups_sub = { $count } w ciągu 14 dni
admin_stat_suspended = Zawieszone
admin_stat_suspended_sub = konta
admin_chart_posts = Wpisy dziennie
admin_chart_signups = Rejestracje dziennie
admin_chart_reports = Zgłoszenia dziennie
admin_chart_heat = Aktywność według dnia tygodnia i godziny
nav_admin = Administracja
admin_title = Administracja
admin_sections = Sekcje administracji
admin_tab_overview = Przegląd
admin_tab_reports = Zgłoszenia
admin_tab_users = Osoby
admin_tab_instance = Instancja
admin_tab_federation = Federacja
admin_tab_announcements = Ogłoszenia
admin_tab_invites = Zaproszenia
admin_tab_filter = Filtr treści
admin_tab_audit = Dziennik audytu
admin_denied_title = Tylko dla personelu
admin_denied_body = Ta część jest przeznaczona dla administratorów i moderatorów tej instancji.
admin_denied_home = Wróć na główną
admin_unknown = nieznane
admin_tag_active = Aktywne
admin_tag_suspended = Zawieszone
admin_tag_deleted = Usunięte
admin_tag_org = Organizacja
admin_audit_system = system
admin_audit_empty = Nic jeszcze nie zapisano
admin_report_status = Stan zgłoszenia
admin_report_open = Otwarte
admin_report_resolved = Rozpatrzone
admin_reports_empty_open = Brak otwartych zgłoszeń
admin_reports_empty_open_body = Gdy ktoś zgłosi wpisy lub konta, pojawią się tutaj.
admin_reports_empty_resolved = Nic jeszcze nie rozpatrzono
admin_report_by = zgłosił(a) { $handle }
admin_report_same_target = Otwarte zgłoszenia: { $count }
admin_report_media =
    { $count ->
        [one] { $count } załącznik
        [few] { $count } załączniki
        [many] { $count } załączników
       *[other] { $count } załącznika
    }
admin_report_poll = Zawiera ankietę
admin_report_dismiss = Odrzuć
admin_report_delete_post = Usuń wpis
admin_report_suspend = Zawieś konto
admin_report_delete_suspend = Usuń i zawieś
admin_report_confirm_title = Podjąć działanie?
admin_report_confirm_body = { $action }. Zostanie to zapisane w dzienniku audytu.
admin_report_confirm = Potwierdź
admin_report_resolution = { $action } przez { $handle } · { $time }
admin_res_dismiss = Odrzucono
admin_res_delete_post = Usunięto wpis
admin_res_suspend_user = Konto zawieszone
admin_res_delete_and_suspend = Wpis usunięto, konto zawieszono
admin_users_search = Szukaj osób
admin_users_search_placeholder = Imię lub nazwa użytkownika
admin_users_show = Pokaż
admin_users_empty = Nie znaleziono osób
admin_filter_local = Lokalne
admin_filter_remote = Zdalne
admin_filter_staff = Personel
admin_filter_verified = Zweryfikowane
admin_filter_suspended = Zawieszone
admin_filter_all = Wszyscy
admin_user_details = Szczegóły: { $handle }
admin_user_menu = Zarządzaj { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } wpis
        [few] { $compact } wpisy
        [many] { $compact } wpisów
       *[other] { $compact } wpisu
    }
admin_user_followers =
    { $count ->
        [one] { $compact } obserwujący
        [few] { $compact } obserwujących
        [many] { $compact } obserwujących
       *[other] { $compact } obserwujących
    }
admin_user_joined = dołączył(a) { $time }
admin_user_active = aktywność { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } otwarte zgłoszenie
        [few] { $compact } otwarte zgłoszenia
        [many] { $compact } otwartych zgłoszeń
       *[other] { $compact } otwartego zgłoszenia
    }
admin_act_verify = Zweryfikuj
admin_act_unverify = Usuń weryfikację
admin_act_org = Zmień w organizację
admin_act_unorg = Usuń status organizacji
admin_act_role_user = Odbierz rolę personelu
admin_act_role_moderator = Nadaj rolę moderatora
admin_act_role_admin = Nadaj rolę administratora
admin_act_suspend = Zawieś
admin_act_unsuspend = Cofnij zawieszenie
admin_act_purge = Usuń wszystkie wpisy
admin_act_rename = Zmień nazwę użytkownika
admin_act_reset_name = Zresetuj wyświetlaną nazwę
admin_act_clear_bio = Wyczyść opis
admin_act_delete = Usuń konto
admin_done_verified = Zweryfikowano { $handle }
admin_done_unverified = Usunięto weryfikację
admin_done_org = { $handle } jest teraz organizacją
admin_done_unorg = Usunięto status organizacji
admin_done_role = Zaktualizowano rolę
admin_done_suspended = Zawieszono { $handle }
admin_done_unsuspended = Cofnięto zawieszenie { $handle }
admin_done_purged =
    { $count ->
        [one] Usunięto { $count } wpis
        [few] Usunięto { $count } wpisy
        [many] Usunięto { $count } wpisów
       *[other] Usunięto { $count } wpisu
    }
admin_done_name_reset = Zresetowano wyświetlaną nazwę
admin_done_bio_cleared = Wyczyszczono opis
admin_done_deleted = Usunięto { $handle }
admin_done_renamed = Teraz: { $handle }
admin_role_phrase_user = zwykłym użytkownikiem
admin_role_phrase_moderator = moderatorem
admin_role_phrase_admin = administratorem
admin_role_title = Zmienić rolę { $handle }?
admin_role_body = Ta osoba zostanie { $role }.
admin_role_confirm = Zmień rolę
admin_unorg_title = Usunąć status organizacji z { $handle }?
admin_unorg_body = Wszystkie jej afiliacje zostaną zakończone.
admin_unorg_confirm = Usuń
admin_suspend_title = Zawiesić { $handle }?
admin_suspend_body = Ta osoba zostanie wylogowana, a jej treści pozostaną ukryte do czasu cofnięcia zawieszenia.
admin_suspend_confirm = Zawieś
admin_purge_title = Usunąć wszystkie wpisy użytkownika { $handle }?
admin_purge_body = Tej operacji nie można cofnąć.
admin_purge_confirm = Usuń wszystkie
admin_reset_name_title = Zresetować wyświetlaną nazwę { $handle }?
admin_reset_name_body = Zostanie zastąpiona nazwą użytkownika.
admin_reset_name_confirm = Resetuj
admin_delete_title = Usunąć { $handle }?
admin_delete_body = Wpisy, obserwowania i wiadomości tej osoby zostaną usunięte, a konto zanonimizowane. Tej operacji nie można cofnąć.
admin_delete_confirm = Usuń konto
admin_rename_title = Zmień nazwę użytkownika { $handle }
admin_rename_note = Linki do starej nazwy użytkownika przestaną działać.
admin_rename_handle = Nowa nazwa użytkownika
admin_rename_reset_name = Zresetuj też wyświetlaną nazwę
admin_rename_save = Zmień nazwę użytkownika
admin_detail_title = Szczegóły konta
admin_detail_invite = Zaproszenie { $code }
admin_detail_posts = Wpisy
admin_detail_followers = Obserwujący
admin_detail_following = Obserwowani
admin_detail_open_reports = Otwarte zgłoszenia
admin_detail_sessions = Sesje
admin_detail_passkeys = Klucze dostępu
admin_detail_tokens = Tokeny API
admin_detail_joined = Dołączył(a) { $time }
admin_detail_recent_posts = Ostatnie wpisy
admin_detail_no_posts = Brak wpisów.
admin_detail_reports = Zgłoszenia dotyczące tego konta
admin_detail_no_reports = Brak zgłoszeń.
admin_detail_history = Historia moderacji
admin_detail_nothing = Na razie nic.
admin_instance_title = Instancja
admin_instance_name = Nazwa instancji
admin_instance_description = Opis
admin_instance_contact = Kontaktowy adres e-mail
admin_instance_contact_hint = Wyświetlany na stronach Regulaminu i Polityki prywatności
admin_instance_min_age = Minimalny wiek
admin_instance_rules = Zasady
admin_instance_rules_hint = Jedna zasada w wierszu
admin_instance_preview = Podgląd regulaminu
admin_instance_save = Zapisz
admin_instance_saved = Zapisano ustawienia instancji
admin_instance_template_note = Regulamin i Polityka prywatności to szablony korzystające z tych wartości. Zanim otworzysz instancję dla wszystkich, zleć ich przegląd pod kątem swojej jurysdykcji.
admin_signups_title = Rejestracje
admin_signups_desc = Kto może założyć konto na tej instancji
admin_signups_open = Otwarte
admin_signups_open_hint = Każdy może założyć konto.
admin_signups_invite = Tylko na zaproszenie
admin_signups_invite_hint = Potrzebny jest kod zaproszenia z karty Zaproszenia.
admin_signups_closed = Zamknięte
admin_signups_closed_hint = Nikt nie może się zarejestrować.
admin_filter_title = Filtr treści
admin_filter_desc = Nic nie jest blokowane, dopóki nie dodasz słów poniżej. Zablokowane słowa dotyczą nazw użytkowników, wyświetlanych nazw, lokalizacji, opisów, wpisów i nazw list, także w zakamuflowanej pisowni. Zarezerwowanych nazw, takich jak „admin”, nie można zarejestrować.
admin_filter_blocked = Zablokowane słowa
admin_filter_blocked_hint = Jedno w wierszu, dopasowywane także wewnątrz słów
admin_filter_allowed = Dozwolone słowa
admin_filter_allowed_hint = Wyjątki od zablokowanych słów, jedno w wierszu
admin_filter_reserved = Dodatkowe zarezerwowane nazwy użytkowników
admin_filter_reserved_hint = Jedna w wierszu
admin_filter_save = Zapisz filtr
admin_filter_saved = Zapisano filtr
admin_filter_field_handle = nazwa użytkownika
admin_filter_field_name = nazwa
admin_filter_field_location = lokalizacja
admin_filter_field_bio = opis
admin_scan_title = Skanuj konta
admin_scan_desc = Znajdź konta, których nazwa użytkownika, nazwa lub opis naruszają filtr.
admin_scan_run = Skanuj teraz
admin_scan_clean = Żadne konto nie narusza filtra.
admin_scan_found =
    { $count ->
        [one] { $count } konto wymaga uwagi
        [few] { $count } konta wymagają uwagi
        [many] { $count } kont wymaga uwagi
       *[other] { $count } konta wymaga uwagi
    }
admin_scan_reason = Pole „{ $field }” zawiera { $reason }
admin_ann_title = Nowe ogłoszenie
admin_ann_desc = Baner na górze ekranu każdego użytkownika, dopóki nie wygaśnie lub go nie zakończysz. Użytkownicy mogą go zamknąć.
admin_ann_text = Wiadomość
admin_ann_link = Link
admin_ann_link_hint = Opcjonalnie: https://… lub /ścieżka
admin_ann_tone = Styl
admin_ann_tone_info = Informacja
admin_ann_tone_success = Dobra wiadomość
admin_ann_tone_warning = Ostrzeżenie
admin_ann_duration = Czas trwania
admin_ann_days =
    { $count ->
        [one] { $count } dzień
        [few] { $count } dni
        [many] { $count } dni
       *[other] { $count } dnia
    }
admin_ann_week = 1 tydzień
admin_ann_until_ended = Do zakończenia
admin_ann_publish = Opublikuj
admin_ann_published = Opublikowano ogłoszenie
admin_ann_list = Dotychczasowe ogłoszenia
admin_ann_none = Brak ogłoszeń.
admin_ann_end = Zakończ
admin_ann_ended = zakończono
admin_ann_expired = wygasło
admin_ann_inactive = Nie jest wyświetlane
admin_ann_until = do { $date }
admin_inv_title = Kody zaproszeń
admin_inv_desc = Pozwól wybranym osobom dołączyć, gdy rejestracja jest tylko na zaproszenie.
admin_inv_note = Notatka (dla kogo)
admin_inv_uses = Użycia
admin_inv_use_count =
    { $count ->
        [one] { $count } użycie
        [few] { $count } użycia
        [many] { $count } użyć
       *[other] { $count } użycia
    }
admin_inv_expiry = Wygasa
admin_inv_week = Za 1 tydzień
admin_inv_days =
    { $count ->
        [one] Za { $count } dzień
        [few] Za { $count } dni
        [many] Za { $count } dni
       *[other] Za { $count } dnia
    }
admin_inv_never = Nigdy
admin_inv_create = Utwórz kod
admin_inv_created = Utworzono kod { $code }
admin_inv_created_copied = Utworzono i skopiowano kod { $code }
admin_inv_list = Kody
admin_inv_none = Brak kodów zaproszeń.
admin_inv_state_active = Aktywny
admin_inv_state_revoked = Unieważniony
admin_inv_state_used = Wykorzystany
admin_inv_state_expired = Wygasły
admin_inv_used = Wykorzystano { $uses } z { $max }
admin_inv_expires = wygasa { $date }
admin_inv_copy = Kopiuj kod { $code }
admin_inv_copied = Skopiowano kod
admin_inv_copy_failed = Nie udało się skopiować. Kod to { $code }.
admin_inv_revoke = Unieważnij
admin_inv_revoke_title = Unieważnić to zaproszenie?
admin_inv_revoke_body = Osoby, które już z niego skorzystały, zachowają swoje konta.
settings_devices = Urządzenia i sesje
settings_devices_desc = Zobacz, gdzie jesteś zalogowany(-a), i wyloguj się zdalnie
settings_passkeys = Klucze dostępu
settings_passkeys_desc = Logowanie odciskiem palca, twarzą, blokadą ekranu lub kluczem bezpieczeństwa
settings_affiliation = Afiliacja
settings_affiliation_desc = Organizacje, do których należysz, lub konta Twojej organizacji
settings_bots = Boty i API
settings_bots_desc = Tokeny API i etykieta „Automatyczne”
settings_saved = Zapisano
passkey_title = Twoje klucze dostępu
passkey_desc = Loguj się odciskiem palca, twarzą, blokadą ekranu lub kluczem bezpieczeństwa zamiast hasłem. Kluczy dostępu nie da się wyłudzić, a na naszym serwerze nie jest przechowywany żaden sekret.
passkey_none = Brak kluczy dostępu.
passkey_add = Dodaj klucz dostępu
passkey_added = Dodano klucz dostępu „{ $name }”
passkey_default_name = Klucz dostępu
passkey_added_ago = dodano { $time }
passkey_used_ago = ostatnio użyty { $time }
passkey_never_used = nigdy nieużywany
passkey_rename = Zmień nazwę
passkey_rename_label = Zmień nazwę klucza dostępu { $name }
passkey_rename_title = Zmień nazwę klucza dostępu
passkey_name = Nazwa
passkey_rename_save = Zapisz
passkey_remove = Usuń
passkey_remove_label = Usuń klucz dostępu { $name }
passkey_remove_title = Usunąć „{ $name }”?
passkey_remove_body = Nie będzie można już logować się tym kluczem dostępu. Możesz też usunąć go z urządzenia lub menedżera haseł.
passkey_removed = Usunięto klucz dostępu
passkey_unsupported = Ta przeglądarka nie obsługuje kluczy dostępu pod tym adresem.
passkey_needs_domain = Klucze dostępu wymagają, aby ta witryna miała nazwę domenową, a nie adres IP.
passkey_already_here = To urządzenie ma już klucz dostępu do tego konta.
token_scope_read = Odczyt
token_scope_write = Publikowanie i interakcje
token_scope_messages = Wiadomości prywatne
token_scope_read_hint = Osie czasu, powiadomienia, profile, wyszukiwanie i strumień na żywo
token_scope_write_hint = Publikowanie, usuwanie, polubienia, podania dalej, zakładki, obserwowania i przesyłanie plików
token_scope_messages_hint = Czytanie i wysyłanie wiadomości prywatnych
token_title = Tokeny API
token_desc = Tokeny pozwalają skryptom i botom korzystać z Twojego konta przez API. Traktuj je jak hasła: każdy z tokenem może działać jako Ty w jego zakresie.
token_none = Brak tokenów.
token_name = Nazwa tokenu
token_name_hint = Na przykład: bot pogodowy
token_scopes = Zakresy
token_create = Utwórz token
token_used_ago = ostatnio użyty { $time }
token_never_used = nigdy nieużywany
token_revoke = Unieważnij
token_revoke_label = Unieważnij token { $name }
token_revoke_title = Unieważnić „{ $name }”?
token_revoke_body = Wszystko, co go używa, przestanie działać.
token_revoked = Unieważniono token
token_secret_title = Twój nowy token
token_secret_warning = Skopiuj go teraz. Ze względów bezpieczeństwa nie zostanie pokazany ponownie.
token_secret_label = Nowy token API
token_copy = Kopiuj
token_copied = Skopiowano token
token_copy_failed = Nie udało się skopiować. Zaznacz token i skopiuj go ręcznie.
token_done = Gotowe
bot_title = Konto automatyczne
bot_desc = Pokazuje etykietę „Automatyczne” obok nazwy, żeby wszyscy wiedzieli, że publikuje bot.
bot_switch = To jest konto automatyczne
bot_owner = Prowadzi
bot_owner_hint = Nazwa użytkownika osoby lub organizacji stojącej za botem. Opcjonalnie.
bot_owner_save = Zapisz
bot_docs = Dokumentacja dla programistów
bot_docs_desc = Endpointy, zdarzenia na żywo i przykładowe boty
aff_yours_title = Twoja afiliacja
aff_yours_desc = Organizacja może zaprosić Cię do wyświetlania jej odznaki obok Twojej nazwy. Nic się nie zmieni, dopóki nie zaakceptujesz, a odejść możesz w każdej chwili.
aff_none = Nie należysz do żadnej organizacji.
aff_invites_title = Zaproszenia
aff_accept = Akceptuj
aff_decline = Odrzuć
aff_leave = Opuść
aff_leave_title = Opuścić tę organizację?
aff_leave_body = Jej odznaka zniknie z Twojego profilu.
aff_remove_title = Usunąć { $handle }?
aff_remove_body = Twoja odznaka zniknie z profilu tej osoby.
aff_remove = Usuń
aff_remove_label = Usuń { $handle }
aff_ended = Zakończono afiliację
aff_declined = Odrzucono zaproszenie
aff_accepted = Dodano afiliację
aff_invited = Wysłano zaproszenie
aff_org_title = Twoi afiliowani
aff_org_desc = Zaproś osoby, aby pokazywały Twoją odznakę obok swojej nazwy. Muszą to zaakceptować.
aff_handle = Nazwa użytkownika
aff_label = Etykieta
aff_label_hint = Opcjonalnie, np. „Inżynier”
aff_invite = Wyślij zaproszenie
aff_members_none = Brak afiliowanych.
aff_pending = Zaproszono
aff_not_org = Jesteś organizacją? Poproś administratora tej instancji o oznaczenie Twojego konta.
auth_passkey = Zaloguj się kluczem dostępu
auth_or = lub
auth_invite = Kod zaproszenia
auth_invite_hint = Ta instancja przyjmuje nowe osoby tylko na zaproszenie.
auth_invite_required = Wpisz otrzymany kod zaproszenia.
landing_developers = Dla programistów
landing_invite = Rejestracja tylko na zaproszenie. Masz kod? Załóż konto.
dev_title = Dla programistów
dev_hero = Boty i API
dev_hero_sub = Twórz boty, skrypty i integracje dla { $name }
dev_intro_h = Twórz dla { $name }
dev_intro_p = Wszystko, co robią aplikacje tuita, przechodzi przez niewielkie API JSON, z którego mogą korzystać także boty i skrypty. Potrzebujesz konta na { $name } i osobistego tokenu API. Nie trzeba rejestrować aplikacji ani przechodzić przez OAuth.
dev_intro_1 = Załóż konto dla swojego bota lub użyj własnego.
dev_intro_2 = W Ustawieniach, w sekcji Boty i API, włącz „To konto automatyczne”, aby ludzie widzieli etykietę „Automatyczne”, i podaj, kto je prowadzi.
dev_intro_3 = Utwórz token z potrzebnymi zakresami i zachowaj go w tajemnicy.
dev_auth_h = Uwierzytelnianie
dev_auth_p = Wysyłaj token w nagłówku Authorization. Każde żądanie i odpowiedź to JSON (UTF-8), poza przesyłaniem plików, które używa multipart.
dev_auth_prefix = Token to tuita_ i 43 litery, cyfry, myślniki i podkreślniki. Przechowywany jest tylko jego skrót, więc zgubionego tokenu nie da się pokazać ponownie: utwórz nowy. Token ma jeden lub więcej z tych zakresów:
dev_auth_refused = Token nie może zmieniać haseł, sesji, tokenów ani kluczy dostępu, eksportować ani usuwać konta, ani korzystać z API administracji. Wymaga to prawdziwego logowania na stronie i kończy się odpowiedzią 403.
dev_auth_errors = Błędy wracają jako JSON z komunikatem error i statusem 4xx. 401 oznacza błędny lub unieważniony token, 403 brak zakresu lub odrzuconą ścieżkę, a 429 przychodzi z nagłówkiem Retry-After.
dev_post_h = Twój pierwszy wpis
dev_post_p = Najpierw prześlij obrazy i filmy przez POST /api/v1/media (multipart), a potem dołącz je po identyfikatorze w mediaIds. Odpowiedź ustawia replyTo, cytat ustawia quoteOf.
dev_endpoints_h = Punkty końcowe
dev_endpoints_intro = Ścieżki w nawiasach klamrowych to symbole zastępcze. Listy stronicuje się przez ?before= ustawione na następną wartość z poprzedniej strony; limit wynosi od 1 do 40. Kolumna zakresu to zakres, którego wymaga token.
dev_col_method = Metoda
dev_col_path = Ścieżka
dev_col_scope = Zakres
dev_col_what = Co robi
dev_ep_me = Konto, do którego należy token
dev_ep_patch_me = Zmień profil i ustawienia tego konta
dev_ep_bot = Włącz lub wyłącz etykietę „Automatyczne” i podaj osobę prowadzącą bota
dev_ep_following = Główna oś czasu osób, które obserwujesz
dev_ep_foryou = Oś czasu „Dla Ciebie”; stronicowanie przez ?cursor=
dev_ep_notifications = Powiadomienia (?filter=mentions dla odpowiedzi, wzmianek i cytatów)
dev_ep_post = Wpis z jego wątkiem i odpowiedziami
dev_ep_user = Profil (handle@domain dla kont na innych serwerach)
dev_ep_user_posts = Czyjeś wpisy (?tab=replies, media lub likes; przy media także ?kind=photo lub video)
dev_ep_search = Szukaj wpisów, osób lub multimediów, z operatorami takimi jak from: i has:media
dev_ep_trends = Najczęściej używane tagi w ostatnim tygodniu
dev_ep_stream = Zdarzenia na żywo (Server-Sent Events)
dev_ep_create = Napisz wpis
dev_ep_thread = Napisz od 2 do 25 wpisów jako wątek
dev_ep_delete = Usuń jeden ze swoich wpisów
dev_ep_like = Polub wpis (DELETE cofa). Także /repost i /bookmark
dev_ep_follow = Obserwuj kogoś (DELETE kończy obserwowanie)
dev_ep_media = Prześlij obraz lub film jako multipart; odpowiedź zawiera identyfikator do dołączenia
dev_ep_schedule = Zaplanuj wpis na później
dev_ep_report = Zgłoś wpis lub konto moderatorom
dev_ep_conversations = Twoje rozmowy z wiadomości prywatnych
dev_ep_message = Wyślij wiadomość prywatną; treść to JSON z polem text
dev_realtime_h = Zdarzenia na żywo
dev_realtime_p = GET /api/v1/stream to strumień Server-Sent Events. Wysyła zdarzenie notification dla każdego polubienia, podania dalej, obserwowania, odpowiedzi, wzmianki lub cytatu, zdarzenie message dla wiadomości prywatnych oraz odznaki z liczbą nieprzeczytanych. Co 25 sekund wysyła wiersz komentarza i prosi klientów o ponowienie po 5 sekundach; połącz się ponownie, jeśli połączenie zostanie przerwane.
dev_example_h = Przykład: bot odpowiadający na wzmianki
dev_example_node = Node 22 lub nowszy, bez pakietów:
dev_example_python = To samo w Pythonie, tylko z biblioteką requests:
dev_limits_h = Limity
dev_limit_rate = Każdy token może wykonać 600 żądań na 10 minut. Po przekroczeniu limitu serwer odpowiada 429 z nagłówkiem Retry-After.
dev_limit_tokens = Konto może mieć do 20 tokenów i tworzyć do 20 tokenów na godzinę.
dev_limit_posting = Publikowanie, obserwowanie, zgłaszanie i inne akcje mają własne limity dla każdego konta.
dev_rules_h = Zasady dla botów
dev_rule_1 = Włącz etykietę Automatyczne i podaj, kto prowadzi bota.
dev_rule_2 = Odpowiadaj lub pisz tylko do osób, które o to poprosiły: wspomniały o bocie, obserwują go albo napisały do niego pierwsze. Nie wysyłaj niechcianych wzmianek ani wiadomości prywatnych.
dev_rule_3 = Zachowaj umiar: trzymaj się dobrze poniżej powyższych limitów.
dev_rule_4 = Przestrzegaj Regulaminu i zasad instancji. Moderatorzy mogą zawieszać boty tak jak każde konto.
admin_act_log_admin_bootstrap = Skonfigurowano pierwszego administratora
admin_act_log_announcement_create = Opublikowano ogłoszenie
admin_act_log_announcement_end = Zakończono ogłoszenie
admin_act_log_instance_settings = Zmieniono ustawienia instancji
admin_act_log_invite_create = Utworzono zaproszenie
admin_act_log_invite_revoke = Unieważniono zaproszenie
admin_act_log_post_delete = Usunięto wpis
admin_act_log_report_dismiss = Odrzucono zgłoszenie
admin_act_log_user_delete = Usunięto konto
admin_act_log_user_org = Oznaczono jako organizację
admin_act_log_user_unorg = Usunięto status organizacji
admin_act_log_user_purge_posts = Usunięto wszystkie wpisy
admin_act_log_user_rename = Zmieniono nazwę konta
admin_act_log_user_role = Zmieniono rolę
admin_act_log_user_suspend = Zawieszono konto
admin_act_log_user_unsuspend = Cofnięto zawieszenie konta
admin_act_log_user_unverify = Usunięto weryfikację
admin_act_log_user_verify = Zweryfikowano konto
admin_act_log_domain_block = Zablokowano instancję
admin_act_log_domain_unblock = Odblokowano instancję
admin_act_log_federation_retry = Zażądano ponowienia dostarczenia
admin_fed_stat_domains = Znane instancje
admin_fed_stat_domains_sub = widziane przez tę instancję
admin_fed_stat_accounts = Konta zdalne
admin_fed_stat_accounts_sub = znane z innych miejsc
admin_fed_stat_posts = Wpisy zdalne
admin_fed_stat_posts_sub = przechowywane lokalnie
admin_fed_stat_queue = W kolejce
admin_fed_stat_queue_sub = oczekujące dostarczenia
admin_fed_queue_title = Kolejka dostarczeń
admin_fed_queue_desc = Podpisane aktywności oczekujące na dotarcie do innych instancji. Nieudane są ponawiane z rosnącym opóźnieniem i porzucane po 12 próbach.
admin_fed_queue_empty = Nic nie czeka. Wszystkie dostarczenia zostały wysłane.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } dostarczenie czeka
        [few] { $compact } dostarczenia czekają
        [many] { $compact } dostarczeń czeka
       *[other] { $compact } dostarczenia czeka
    }
admin_fed_queue_due = Do wysłania teraz: { $count }
admin_fed_queue_oldest = Najstarsze w kolejce od { $date }
admin_fed_retry = Ponów teraz
admin_fed_retry_done =
    { $count ->
        [one] { $compact } dostarczenie zostanie teraz ponowione.
        [few] { $compact } dostarczenia zostaną teraz ponowione.
        [many] { $compact } dostarczeń zostanie teraz ponowionych.
       *[other] { $compact } dostarczenia zostanie teraz ponowione.
    }
admin_fed_failing = Instancje z błędami
admin_fed_failing_none = Żadna instancja nie zgłasza błędów.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } dostarczenie
        [few] { $compact } dostarczenia
        [many] { $compact } dostarczeń
       *[other] { $compact } dostarczenia
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } próba
        [few] { $compact } próby
        [many] { $compact } prób
       *[other] { $compact } próby
    }
admin_fed_failing_next = następna próba { $date }
admin_fed_failing_error = Ostatni błąd
admin_fed_failing_no_error = Nie zapisano błędu
admin_fed_actor_title = Aktor instancji
admin_fed_actor_desc = Inne instancje sprawdzają żądania podpisywane przez tę instancję względem tego klucza.
admin_fed_actor_id = Aktor
admin_fed_actor_key = Identyfikator klucza
admin_fed_actor_fingerprint = Odcisk SHA-256
admin_fed_actor_copy = Kopiuj odcisk
admin_fed_actor_copied = Skopiowano odcisk
admin_fed_actor_copy_failed = Nie udało się skopiować. Odcisk to { $fingerprint }
admin_fed_blocks_title = Zablokowane instancje
admin_fed_blocks_desc = Wyciszenie ukrywa konta instancji na publicznych osiach czasu i w wyszukiwaniu. Zawieszenie ukrywa je wszędzie, odrzuca to, co wysyłają, i wstrzymuje dostarczanie do nich.
admin_fed_domain = Domena
admin_fed_domain_hint = example.social lub link do dowolnej strony na niej
admin_fed_severity = Poziom
admin_fed_silence = Wycisz
admin_fed_suspend = Zawieś
admin_fed_reason = Powód
admin_fed_reason_hint = Opcjonalnie, widoczny tylko dla personelu
admin_fed_block = Zablokuj instancję
admin_fed_blocked = Zablokowano { $domain }
admin_fed_none = Żadna instancja nie jest zablokowana.
admin_fed_list = Blokady
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } znane konto
        [few] { $compact } znane konta
        [many] { $compact } znanych kont
       *[other] { $compact } znanego konta
    }
admin_fed_row_no_reason = Nie podano powodu
admin_fed_row_changed = { $domain } ma teraz poziom: { $severity }
admin_fed_remove = Usuń
admin_fed_remove_label = Zdejmij blokadę z { $domain }
admin_fed_remove_title = Zdjąć blokadę z { $domain }?
admin_fed_remove_body = Jej konta i wpisy wrócą, a dostarczanie do niej zostanie wznowione.
admin_fed_removed = Odblokowano { $domain }
remote_domain_title = Konto na { $domain }
remote_view_original = Zobacz na { $domain }
visibility_unlisted = Poza publiczną osią czasu
visibility_unlisted_hint = Nie jest pokazywany na publicznych osiach czasu
visibility_followers = Tylko obserwujący
visibility_followers_hint = Tylko obserwujący mogą to zobaczyć
visibility_direct = Bezpośredni
visibility_direct_hint = Tylko wspomniane osoby mogą to zobaczyć
follow_requested = Wysłano prośbę
follow_cancel_request = Anuluj prośbę o obserwowanie
follow_requested_toast = Wysłano prośbę o obserwowanie do { $handle }
follow_cancelled_toast = Anulowano prośbę o obserwowanie
profile_locked = Zatwierdza obserwujących
profile_locked_hint = Prośby o obserwowanie wymagają zatwierdzenia przez to konto.
profile_original = Zobacz profil na { $domain }
profile_remote_note = Kopia z { $domain }. Liczby i wpisy to to, co widziała ta instancja.
remote_lookup_loading = Wyszukiwanie { $handle } na { $domain }…
remote_lookup_notfound_title = Nie ma takiego konta
remote_lookup_notfound_body = { $handle } nie istnieje albo { $domain } go nie udostępnia.
remote_lookup_failed_title = Nie można wczytać { $handle }
explore_lookup = Wyszukiwanie { $target }…
explore_lookup_empty_body = Nie znaleziono tego konta ani wpisu. Inna instancja może być wyłączona albo go nie udostępnia.
language_label = Język
oauth_title = Autoryzuj aplikację
oauth_loading = Sprawdzanie żądania…
oauth_wants = Ta aplikacja chce używać Twojego konta { $handle }.
oauth_name_warning = Każdy może wybrać dowolną nazwę aplikacji. Zatwierdzaj tylko aplikacje, którym ufasz.
oauth_website = Strona internetowa
oauth_allow = Aplikacja będzie mogła
oauth_scope_read = Czytać Twój profil, wpisy, obserwowania, listy, osie czasu i powiadomienia oraz wiadomości prywatne, które nie są szyfrowane end-to-end.
oauth_scope_write = Publikować, edytować i usuwać w Twoim imieniu, dawać polubienia, obserwować, blokować, wyciszać oraz zmieniać Twój profil i ustawienia.
oauth_scope_messages = Czytać i wysyłać Twoje wiadomości prywatne.
oauth_push_note = Powiadomienia push nie są obsługiwane. Aplikacja, która o nie prosi, ich nie otrzyma.
oauth_redirect = Po Twojej odpowiedzi nastąpi przekierowanie do
oauth_redirect_oob = Aplikacja nie otrzyma przekierowania. Kod pokażemy tutaj, abyś wkleił(a) go w aplikacji.
oauth_approve = Autoryzuj
oauth_deny = Odmów
oauth_switch = Użyj innego konta
oauth_sent_approved = Autoryzowano. Wracamy do { $host }. Jeśli nic się nie stanie, zamknij tę kartę.
oauth_sent_denied = Odmówiono. Wracamy do { $host }. Jeśli nic się nie stanie, zamknij tę kartę.
oauth_denied = Odmówiono. Aplikacja nic nie otrzymała.
oauth_code_title = Kod autoryzacji
oauth_code_body = Wklej ten kod w aplikacji. Działa raz i wygasa po 10 minutach.
oauth_code_copy = Kopiuj kod
oauth_code_copied = Skopiowano kod
oauth_code_copy_failed = Nie udało się skopiować. Zaznacz kod i skopiuj go ręcznie.
oauth_error_title = Nie można autoryzować tego żądania
oauth_error_note = Do aplikacji nic nie wysłano.
oauth_error_invalid_request = Żądanie jest nieprawidłowe albo prosi o powrót pod adres, którego aplikacja nie zarejestrowała. Wróć do aplikacji i spróbuj ponownie.
oauth_error_invalid_client = Ta aplikacja nie jest tu zarejestrowana.
oauth_error_invalid_scope = Aplikacja prosi o dostęp, którego tuita nie oferuje, albo o więcej, niż zarejestrowała.
oauth_error_unsupported_response_type = Aplikacja prosi o rodzaj logowania, którego tuita nie obsługuje.
oauth_error_rate = Odpowiedziano na zbyt wiele żądań. Odczekaj kilka minut i spróbuj ponownie.
oauth_error_unreachable = Nie można połączyć się z tuita. Sprawdź połączenie i spróbuj ponownie.
oauth_home = Przejdź do tuita
share_send_chat = Wyślij na czacie
share_download = Pobierz multimedia
share_download_n = Pobierz multimedia { $number }
share_chat_title = Wyślij post na czacie
share_chat_sent = Wysłano do { $handle }
hover_card_label = Karta profilu: { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Rodzaj multimediów
profile_media_all = Wszystkie multimedia
profile_media_photos = Zdjęcia
profile_media_videos = Filmy
profile_empty_videos_title = Brak filmów
profile_empty_videos_body = Tutaj pojawią się posty z filmami.
account_add_existing = Dodaj istniejące konto
rail_label = Wyszukiwanie i propozycje
rail_search_submit = Szukaj
rail_show_more = Pokaż więcej
rail_footer = Linki
chat_new = Nowy czat
chat_search = Szukaj rozmów
chat_search_none = Żadna rozmowa nie pasuje do wyszukiwania.
chat_start_title = Rozpocznij rozmowę
chat_start_body = Wybierz rozmowę z listy lub zacznij nową.
explore_sections = Sekcje Odkrywaj
explore_see_more = Pokaż więcej
history_title = Historia
history_tabs = Sekcje historii
history_likes_label = Polubione posty
video_play = Odtwórz
video_pause = Wstrzymaj
video_mute = Wycisz
video_unmute = Włącz dźwięk
video_volume = Głośność
video_seek = Przewiń
video_position = { $current } z { $total }
video_speed = Szybkość odtwarzania
video_fullscreen = Pełny ekran
video_exit_fullscreen = Zamknij pełny ekran
video_pip = Obraz w obrazie
video_exit_pip = Zamknij obraz w obrazie
video_error = Nie można odtworzyć tego filmu.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Ukryj panel
lightbox_show_panel = Pokaż panel
lightbox_view_post = Zobacz post
lightbox_panel = Post i odpowiedzi
emotepick_tabs = Zestawy emoji
emotepick_tab_emoji = Standardowe
emotepick_tab_custom = Własne
admin_emote_tab = Emotki
admin_emote_title = Własne emotki
admin_emote_desc = Emotki należą do całej instancji. Każdy może ich używać jako :shortcode: we wpisach, nazwach i wiadomościach. Kodu emotki nie można zmienić; usuń emotkę i prześlij ją ponownie.
admin_emote_shortcode = Kod
admin_emote_shortcode_hint = Od 2 do 32 liter, cyfr lub podkreślników. Wyświetlany jako :shortcode:.
admin_emote_invalid = Użyj tylko liter, cyfr i podkreślników, od 2 do 32 znaków.
admin_emote_category = Kategoria
admin_emote_category_hint = Opcjonalna. Grupuje emotki w selektorze.
admin_emote_file = Obraz emotki
admin_emote_file_hint = PNG, JPEG, WebP lub GIF, do 256 KB. Upuść plik tutaj lub wybierz go.
admin_emote_file_type = Ten plik nie jest obrazem PNG, JPEG, WebP ani GIF.
admin_emote_file_size = Obraz jest za duży (maks. 256 KB).
admin_emote_choose_file = Wybierz obraz
admin_emote_change_file = Zmień obraz
admin_emote_upload = Prześlij emotkę
admin_emote_upload_failed = Nie udało się przesłać emotki. Spróbuj ponownie.
admin_emote_uploaded = Dodano :{ $shortcode }:.
admin_emote_list = Emotki w tej instancji
admin_emote_none = Brak własnych emotek.
admin_emote_no_category = Bez kategorii
admin_emote_category_for = Kategoria :{ $shortcode }:
admin_emote_saved = Zapisano kategorię.
admin_emote_delete = Usuń :{ $shortcode }:
admin_emote_delete_title = Usunąć tę emotkę?
admin_emote_delete_body = :{ $shortcode }: przestaje być wszędzie wyświetlana jako obraz i pojawia się jako zwykły tekst. Wpisy, które jej używały, zachowują tekst.
admin_emote_delete_confirm = Usuń emotkę
admin_emote_deleted = Usunięto emotkę.
admin_act_log_emoji_create = Dodano emotkę
admin_act_log_emoji_update = Zmieniono emotkę
admin_act_log_emoji_delete = Usunięto emotkę
e2ee_settings = Szyfrowane wiadomości
e2ee_settings_desc = Fraza hasłowa, to urządzenie i reset
e2ee_status_label = Stan
e2ee_status_none = Wyłączone. Włącz, aby wysyłać i czytać wiadomości szyfrowane end-to-end.
e2ee_status_locked = Zablokowane na tym urządzeniu. Wpisz frazę hasłową, aby czytać tu swoje wiadomości.
e2ee_status_ready = Włączone i odblokowane na tym urządzeniu.
e2ee_keys_error = Nie udało się sprawdzić Twoich kluczy szyfrowania.
e2ee_setup_title = Włącz szyfrowane wiadomości
e2ee_setup_body = Wiadomości do osób z tego serwera są szyfrowane w Twojej przeglądarce. Serwer przechowuje tylko tekst, którego nie może odczytać. Fraza hasłowa pozwala odblokować wiadomości na innym urządzeniu.
e2ee_setup_warning = Frazy hasłowej nie da się odzyskać. Jeśli ją zapomnisz, możesz zresetować szyfrowanie, ale wcześniejsze wiadomości przepadną.
e2ee_setup_submit = Włącz
e2ee_setup_done = Szyfrowane wiadomości są włączone
e2ee_unlock_title = Odblokuj szyfrowane wiadomości
e2ee_unlock_body = Wpisz frazę hasłową, aby czytać i wysyłać szyfrowane wiadomości na tym urządzeniu.
e2ee_unlock_submit = Odblokuj
e2ee_unlock_wrong = Ta fraza hasłowa jest nieprawidłowa.
e2ee_unlock_done = Odblokowano
e2ee_passphrase = Fraza hasłowa
e2ee_passphrase_confirm = Potwierdź frazę hasłową
e2ee_passphrase_hint = Co najmniej 10 znaków.
e2ee_passphrase_short = Użyj co najmniej 10 znaków.
e2ee_passphrase_mismatch = Frazy hasłowe nie są takie same.
e2ee_failed = Coś poszło nie tak. Spróbuj ponownie.
e2ee_unsupported = Ta przeglądarka nie obsługuje szyfrowanych wiadomości.
e2ee_forget = Zapomnij na tym urządzeniu
e2ee_forget_hint = Usuwa z tej przeglądarki Twój klucz i zaufane kontakty. Fraza hasłowa odblokuje je ponownie.
e2ee_forget_done = Zapomniano na tym urządzeniu
e2ee_reset = Zresetuj szyfrowanie
e2ee_reset_hint = Tworzy nowy klucz. Wiadomości wysłane i odebrane wcześniej staną się nieczytelne.
e2ee_reset_title = Zresetować szyfrowanie?
e2ee_reset_body = Wcześniejsze wiadomości staną się nieczytelne, dla Ciebie i dla Twoich rozmówców, i nie da się ich przywrócić. Twoje kontakty zostaną poproszone o zaufanie nowemu kluczowi. Wybierz nową frazę hasłową.
e2ee_reset_submit = Zresetuj
e2ee_reset_done = Szyfrowanie zostało zresetowane
e2ee_note_sealed = Szyfrowane end-to-end
e2ee_note_remote = Brak szyfrowania end-to-end: ta osoba jest na innym serwerze.
e2ee_note_plain = Brak szyfrowania end-to-end.
e2ee_plain_mark = Bez szyfrowania
e2ee_preview = Szyfrowana wiadomość
e2ee_line_sealed = Szyfrowana wiadomość
e2ee_line_own_key = Wysłano do klucza, którego już nie masz
e2ee_line_peer_key = Wysłano kluczem, któremu jeszcze nie ufasz
e2ee_line_unreadable = Nie udało się otworzyć
e2ee_line_unsupported = Ta przeglądarka nie może otwierać szyfrowanych wiadomości
e2ee_block_peer_off = Użytkownik { $handle } nie ma jeszcze włączonych szyfrowanych wiadomości.
e2ee_block_self_off = Najpierw włącz szyfrowane wiadomości w Ustawieniach.
e2ee_block_unlock = Najpierw odblokuj szyfrowane wiadomości w Ustawieniach.
e2ee_block_downgrade = Użytkownik { $handle } używał już szyfrowanych wiadomości. Ta rozmowa nie wróci do wiadomości bez szyfrowania.
e2ee_block_changed = Zaufaj nowemu kluczowi użytkownika { $handle }, aby pisać do tej osoby.
e2ee_block_error = Nie udało się sprawdzić Twoich kluczy szyfrowania.
e2ee_changed_title = Klucz szyfrowania użytkownika { $name } się zmienił
e2ee_changed_body = Ta osoba mogła zresetować szyfrowanie lub przejść na nowe urządzenie albo ktoś wchodzi między was. Porównaj z nią nowy numer bezpieczeństwa, zanim mu zaufasz. Do tego czasu nie możesz wysyłać wiadomości.
e2ee_changed_old = Dotąd zaufany numer bezpieczeństwa
e2ee_changed_new = Nowy numer bezpieczeństwa
e2ee_trust = Zaufaj nowemu kluczowi
e2ee_safety_open = Numer bezpieczeństwa
e2ee_safety_title = Numer bezpieczeństwa
e2ee_safety_body = Porównaj ten numer z użytkownikiem { $name } osobiście lub podczas rozmowy, nie na tym czacie. Jeśli na obu ekranach jest taki sam, nikt nie wchodzi między was.

## 0031
composer_upload_failed = Przesyłanie nie powiodło się. Spróbuj ponownie.
composer_send_failed = Nie można połączyć się z tuita. Sprawdź połączenie i spróbuj ponownie.
composer_open_failed = Nie udało się otworzyć edytora wpisu. Spróbuj ponownie.
