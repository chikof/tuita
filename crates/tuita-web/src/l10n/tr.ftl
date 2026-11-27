app_name = tuita
loading = Yükleniyor
close = Kapat
dismiss = Kapat
notifications = Bildirimler
avatar_more = { $count } kişi daha
char_counter_left =
    { $count ->
        [one] { $count } karakter kaldı
       *[other] { $count } karakter kaldı
    }
char_counter_over =
    { $count ->
        [one] Sınırı { $count } karakter aştın
       *[other] Sınırı { $count } karakter aştın
    }
not_found_title = Burada bir şey yok
not_found_body = Bu sayfa yok ya da taşınmış.
not_found_back = Geri dön
tagline = Her gönderi, iki mürekkep.
error_title = Bir şeyler ters gitti
cancel = İptal
retry = Tekrar dene
back = Geri
view = Görüntüle
follow = Takip et
following = Takip edilenler
follows_you = Seni takip ediyor
verified = Doğrulanmış hesap
bot_account = Otomatik hesap
bot_label = Bot
link_copied = Bağlantı panoya kopyalandı
text_copied = Metin kopyalandı
time_now = şimdi
time_minutes = { $n } dk
time_hours = { $n } sa
time_days = { $n } g
nav_main = Ana menü
nav_home = Ana sayfa
nav_explore = Keşfet
nav_notifications = Bildirimler
nav_bookmarks = Yer işaretleri
nav_profile = Profil
nav_settings = Ayarlar
nav_compose = Gönder
nav_menu = Menü
badge_unread = { $count } okunmamış
account_menu = Hesaplar
account_switch = Hesap değiştir
account_switch_to = { $handle } hesabına geç
account_add = Hesap ekle
account_logout = { $handle } hesabından çık
account_logout_all = Tüm hesaplardan çık
home_feed = Zaman tüneli
home_for_you = Sana özel
home_following = Takip edilenler
home_empty_title = Burada henüz bir şey yok
home_empty_body = İlk gönderiyi sen paylaş.
home_following_empty_title = Çevren sessiz
home_following_empty_body = Takip ettiğin kişilerin gönderileri ve yeniden paylaşımları burada görünecek. Birilerini takip etmeye başla.
home_find_people = Kişi bul
new_posts =
    { $count ->
        [one] { $count } yeni gönderiyi göster
       *[other] { $count } yeni gönderiyi göster
    }
new_posts_many = 99+ yeni gönderiyi göster
composer_title = Yeni gönderi
composer_reply_title = Yanıtla
composer_quote_title = Gönderiyi alıntıla
composer_placeholder = Neler oluyor?
composer_reply_placeholder = Yanıtını gönder
composer_text_label = Gönderi metni
composer_replying_to = { $handle } kişisine yanıt
composer_cw_label = İçerik uyarısı
composer_cw_placeholder = İçerik uyarısı, örn. spoiler
composer_cw_toggle = İçerik uyarısı ekle
composer_sensitive_toggle = Medyayı hassas olarak işaretle
composer_options = Gönderi seçenekleri
composer_policy_label = Kimler yanıtlayabilir
composer_suggestions = Öneriler
composer_tag_posts =
    { $count ->
        [one] { $compact } gönderi
       *[other] { $compact } gönderi
    }
composer_post = Gönder
composer_reply = Yanıtla
policy_everyone = Herkes yanıtlayabilir
policy_following = Takip ettiklerin yanıtlayabilir
policy_mentioned = Yalnızca bahsettiklerin yanıtlayabilir
post_sent = Gönderi paylaşıldı
post_by = { $name } kişisinin gönderisi
post_reposted_by = { $name } yeniden paylaştı
post_pinned = Sabitlenmiş gönderi
post_replying_to = { $handle } kişisine yanıt
post_replying_unavailable = Kullanılamayan bir gönderiye yanıt
post_unavailable = Bu gönderi kullanılamıyor.
post_menu = Daha fazla
post_copy_link = Bağlantıyı kopyala
post_copy_text = Metni kopyala
post_pin = Profile sabitle
post_unpin = Profilden sabitlemeyi kaldır
post_pinned_toast = Profiline sabitlendi
post_unpinned = Profilinden kaldırıldı
post_delete = Gönderiyi sil
post_delete_title = Gönderi silinsin mi?
post_delete_body = Bu işlem geri alınamaz. Gönderi profilinden, takipçilerinin zaman tünellerinden ve aramadan kaldırılır.
post_deleted = Gönderi silindi
post_view_profile = { $handle } profilini gör
post_mute = { $handle } kişisini sessize al
post_block = { $handle } kişisini engelle
post_reply_label = Yanıtla ({ $count })
post_reply_following_only = Yalnızca yazarın takip ettiği kişiler yanıt verebilir.
post_reply_mentioned_only = Yalnızca bu gönderide bahsedilen kişiler yanıt verebilir.
post_repost_menu = Yeniden paylaşım seçenekleri
post_repost = Yeniden paylaş
post_undo_repost = Yeniden paylaşımı geri al
post_quote = Alıntıla
post_reposts_label = Yeniden paylaşımlar ve alıntılar ({ $count })
post_like_label = Beğen ({ $count })
post_bookmark_label = Yer işareti ekle
post_share = Paylaş veya bağlantıyı kopyala
cw_show = Göster
cw_hide = Gizle
cw_label = İçerik uyarısı: { $reason }
muted_toast = { $handle } sessize alındı
unmuted_toast = { $handle } kişisinin sesi açıldı
blocked_toast = { $handle } engellendi
unblocked_toast = { $handle } kişisinin engeli kaldırıldı
block_title = { $handle } engellensin mi?
block_body = Seni takip edemez ve onun gönderilerini görmezsin. Kendisine haber verilmez.
block_confirm = Engelle
notify_like = @{ $handle } gönderini beğendi
notify_repost = @{ $handle } gönderini yeniden paylaştı
notify_reply = @{ $handle } sana yanıt verdi
notify_mention = @{ $handle } senden bahsetti
notify_quote = @{ $handle } gönderini alıntıladı
notify_follow = @{ $handle } seni takip etti
notify_follow_request = @{ $handle } sana takip isteği gönderdi
notify_poll = Oy verdiğin bir anket sona erdi
notify_affiliation = @{ $handle } seni bağlı hesap olmaya davet etti
shortcuts_title = Klavye kısayolları
shortcut_group_navigation = Gezinme
shortcut_group_actions = İşlemler
shortcut_or = /
shortcut_then = sonra
shortcut_plus = +
shortcut_next_prev = Sonraki / önceki gönderi
shortcut_open = Gönderiyi aç
shortcut_search = Ara
shortcut_go_home = Ana sayfa
shortcut_go_explore = Keşfet
shortcut_go_notifications = Bildirimler
shortcut_go_bookmarks = Yer işaretleri
shortcut_go_profile = Profilin
shortcut_go_settings = Ayarlar
shortcut_new_post = Yeni gönderi
shortcut_like = Beğen
shortcut_reply = Yanıtla
shortcut_repost = Yeniden paylaş veya alıntıla
shortcut_bookmark = Yer işareti ekle
shortcut_send = Gönderiyi paylaş
shortcut_help = Bu yardımı göster
auth_login_title = Giriş yap
auth_add_title = Başka bir hesap ekle
auth_add_sub = Bu cihazda çıkış yapmadan en fazla { $max } hesap arasında geçiş yapmak için giriş yap.
auth_signup_title = Hesap oluştur
auth_handle = Kullanıcı adı
auth_password = Şifre
auth_display_name = Görünen ad
auth_handle_hint = 3–15 harf, rakam veya alt çizgi
auth_password_hint = En az 8 karakter
auth_consent_pre = En az { $minAge } yaşındayım ve şunları kabul ediyorum:{" "}
auth_consent_terms = Hizmet Şartları
auth_consent_and = {" "}ve{" "}
auth_consent_privacy = Gizlilik Politikası
auth_consent_end = {"."}
auth_consent_required = Lütfen Hizmet Şartları'nı ve Gizlilik Politikası'nı kabul et.
auth_signin = Giriş yap
auth_signin_add = Giriş yap ve geç
auth_create = Hesap oluştur
auth_to_signup = Yeni misin? Hesap oluştur
auth_to_login = Hesabın var mı? Giriş yap
auth_closed = Bu sunucuda kayıtlar şu anda kapalı.
landing_nav = Alt bilgi
landing_rules = Sunucu kuralları
landing_stats = Şu anda
landing_people = Kişiler
landing_posts = Gönderiler
landing_clock = Yerel saatin
landing_trends = Gündemde
landing_trend =
    { $count ->
        [one] { $count } gönderi
       *[other] { $count } gönderi
    }
landing_join = { $name } sunucusuna katıl
landing_join_sub = Hesap oluştur ve iki mürekkeple paylaş.
landing_closed = Kayıtlar şu anda kapalı. Zaten hesabın var mı? Giriş yap.
landing_terms = Şartlar
landing_privacy = Gizlilik
landing_source = Kaynak kodu
landing_powered = tuita { $version } ile çalışıyor
docs_terms_title = Hizmet Şartları
docs_privacy_title = Gizlilik Politikası
docs_updated = Son güncelleme: { $date }
docs_date = 5 Kasım 2026
docs_see_privacy = Gizlilik Politikası'na da bak.
docs_see_terms = Hizmet Şartları'na da bak.
docs_contact_fallback = sunucu yöneticileri (doğrudan mesaj veya şikâyet yoluyla)
docs_terms_agreement_h = 1. Anlaşma
docs_terms_agreement_p = Bu Hizmet Şartları (“Şartlar”), { $domain } adresindeki tuita sunucusu { $name } hizmetini (“hizmet”, “biz”) kullanmanı düzenler. Hesap oluşturarak veya hizmeti kullanarak bu Şartları ve Gizlilik Politikamızı kabul etmiş olursun. Kabul etmiyorsan hizmeti kullanma.
docs_terms_eligibility_h = 2. Kimler kullanabilir
docs_terms_eligibility_p = En az { $minAge } yaşında olmalısın; yaşadığın yerde dijital rıza yaşı daha yüksekse o yaşta olmalısın. Bağlayıcı bir anlaşma yapabilecek durumda olmalı ve geçerli yasalar uyarınca hizmeti kullanman yasaklanmamış olmalı.
docs_terms_account_h = 3. Hesabın
docs_terms_account_1 = Şifreni gizli tut. Hesabındaki etkinliklerden sen sorumlusun.
docs_terms_account_2 = Kullanıcı adın başka bir kişiyi veya kuruluşu taklit etmemeli, kim olduğun konusunda insanları yanıltmamalı.
docs_terms_account_3 = Hesabının ele geçirildiğini düşünüyorsan bize hemen bildir. Diğer oturumları Ayarlar → Hesap bölümünden kapatabilirsin.
docs_terms_content_h = 4. İçeriğin
docs_terms_content_p1 = Paylaştıklarının sahibi sen olmaya devam edersin. Bize, içeriğini yalnızca hizmeti işletmek ve geliştirmek için saklama, gösterme, çoğaltma ve dağıtma konusunda dünya çapında, münhasır olmayan ve telifsiz bir lisans verirsin. Buna içeriğini federasyondaki tuita sunucularına göndermek de dahildir (6. bölüme bak). Bu lisans, içeriğini veya hesabını sildiğinde sona erer; ancak diğer sunuculara ulaşmış veya başkaları tarafından kaydedilmiş kopyalar bunun dışındadır.
docs_terms_content_p2 = Paylaştıklarından sen sorumlusun. Yalnızca paylaşma hakkına sahip olduğun içerikleri paylaş.
docs_terms_rules_h = 5. Topluluk kuralları
docs_terms_rules_lead = { $name } kullanırken yasaları veya şu kuralları çiğnememeyi kabul edersin:
docs_terms_rules_p = Ayrıca hizmete saldırmamayı da kabul edersin; örneğin güvenlik önlemlerini aşmaya çalışmak, büyük ölçekte veri kazımak, kötü amaçlı yazılım göndermek veya hizmeti aşırı yüklemek gibi.
docs_terms_federation_h = 6. Federasyon
docs_terms_federation_p = { $name }, yöneticilerin onayladığı diğer bağımsız tuita sunucularıyla bağlantı kurabilir (“federasyon”). Başka bir sunucudaki kişilerle etkileşime girdiğinde herkese açık profilin, herkese açık gönderilerin ve etkileşim (örneğin takip, beğeni, yeniden paylaşım, yanıt veya doğrudan mesaj) o sunucuya gönderilir. Diğer sunucuları başkaları kendi şartlarıyla işletir. İçerik başka bir sunucuya ulaştıktan sonra onu kontrol edemeyiz veya silemeyiz; ancak bir şeyi sildiğinde onlara silme isteği göndeririz.
docs_terms_moderation_h = 7. Moderasyon ve uygulama
docs_terms_moderation_p = Moderatörler, bu Şartları veya yasaları ihlal eden içerikleri kaldırabilir, özellikleri sınırlayabilir veya hesapları askıya alabilir. Mümkün olduğunda nedenini sana bildiririz. Herkes bir gönderiyi veya hesabı şikâyet edebilir. Şikâyetler bu sunucunun moderatörlerine gider. Herhangi bir sunucuyla federasyonu istediğimiz zaman durdurabiliriz.
docs_terms_termination_h = 8. Ayrılma
docs_terms_termination_p = Verilerini dışa aktarabilir veya hesabını istediğin zaman Ayarlar → Hesap bölümünden silebilirsin. Bu Şartları ciddi biçimde veya tekrar tekrar ihlal edersen erişimini askıya alabilir veya sonlandırabiliriz. Hizmeti kapatabiliriz de; bunu yapmadan önce makul bir bildirim vermeye çalışırız.
docs_terms_disclaimers_h = 9. Sorumluluk reddi ve yükümlülük
docs_terms_disclaimers_p = Hizmet, yasaların izin verdiği en geniş ölçüde, hiçbir türde garanti verilmeksizin “olduğu gibi” ve “mevcut haliyle” sunulur. Dolaylı, arızi veya sonuç olarak ortaya çıkan zararlardan ya da kullanıcıların veya diğer sunucuların paylaştığı içeriklerden sorumlu değiliz. Bu Şartlardaki hiçbir hüküm, yasayla sınırlanamayacak yükümlülüğü veya tüketici olarak sahip olduğun yasal hakları sınırlamaz.
docs_terms_changes_h = 10. Değişiklikler
docs_terms_changes_p = Bu Şartları güncelleyebiliriz. Değişiklik önemliyse yürürlüğe girmeden önce uygulama içinde bildirim yaparız. Bundan sonra hizmeti kullanmaya devam etmen yeni Şartları kabul ettiğin anlamına gelir.
docs_terms_contact_h = 11. İletişim
docs_terms_contact_lead = Bu Şartlarla ilgili sorular için:
docs_privacy_summary_h = Kısa özet
docs_privacy_summary_1 = Reklam yok, izleme betiği yok, veri satışı yok.
docs_privacy_summary_2 = Oturumunu açık tutan tek bir çerez var. Tema tercihlerin tarayıcında kalır.
docs_privacy_summary_3 = Herkese açık gönderiler herkese açıktır ve federasyondaki tuita sunucularına ulaşabilir.
docs_privacy_summary_4 = Bu sunucudaki hesaplar arasındaki doğrudan mesajlar, iki taraf da açtığında uçtan uca şifrelenir: sunucunun okuyamadığı metin olarak saklanırlar, bu yüzden bir veritabanı sızıntısı ya da saklanan verilere bakmak hiçbir şey ortaya çıkarmaz. Başka sunuculardaki kişilere giden mesajlar ile şifrelemeyi açmadan önce gönderilen mesajlar bu şekilde şifrelenmez ve yöneticiler bunları teknik olarak okuyabilir. Bu sunucu şifrelemeyi yapan uygulamayı da sunar; dolayısıyla bu uygulamayı değiştiren bir yönetici yazdıklarını yakalayabilir.
docs_privacy_summary_5 = Verilerini istediğin zaman dışa aktarabilir veya silebilirsin.
docs_privacy_controller_h = 1. Biz kimiz
docs_privacy_controller_p = Bu politika, sunucu yöneticileri tarafından işletilen { $domain } adresindeki { $name } hizmetini kapsar. Verilerinin nasıl işleneceğine onlar karar verir.
docs_privacy_collect_h = 2. Ne topluyoruz
docs_privacy_collect_1 = Hesap verileri: kullanıcı adı, görünen ad, şifre (yalnızca tuzlanmış argon2id özeti olarak saklanır) ve profil ayrıntıların: biyografi, konum, web sitesi, avatar, kapak görseli, şekil ve renk.
docs_privacy_collect_2 = İçerik: gönderiler, görseller, yanıtlar, beğeniler, yeniden paylaşımlar, yer işaretleri, takipler, engeller, sessize almalar ve şikâyetler.
docs_privacy_collect_3 = Doğrudan mesajlar: sana ve diğer kişiye iletilebilmesi için bu sunucuda saklanır. Uçtan uca şifreli bir mesaj, okuyamadığımız şifreli metin olarak saklanır; kimin kime ne zaman gönderdiği bizim tarafımızdan görülebilir. Diğer mesajlar düz metin olarak saklanır. Şifreleme anahtarın burada yalnızca, bizim hiç görmediğimiz şifre cümlenle kilitlenmiş biçimde saklanır.
docs_privacy_collect_4 = Oturum verileri: oturumları gözden geçirip kapatabilmen için oturum açılan her cihazın tarayıcı kullanıcı aracısı, IP adresi ve son kullanım zamanı.
docs_privacy_collect_5 = Kısa ömürlü güvenlik verileri: hız sınırlaması için birkaç dakika boyunca bellekte tutulan IP adresleri.
docs_privacy_collect_p = Profiline kendin yazmadıkça e-postanı, telefon numaranı, kişilerini veya konumunu toplamayız.
docs_privacy_use_h = 3. Nasıl kullanıyoruz
docs_privacy_use_p = Yalnızca hizmeti işletmek için: profilini ve gönderilerini göstermek, zaman tünellerini ve bildirimleri oluşturmak, mesajları iletmek, hesapları güvende tutmak, kötüye kullanımı önlemek ve Şartları uygulamak. “Sana özel” zaman tüneli gönderileri beğeni, yeniden paylaşım, yanıt ve yaşa göre sıralar. Seni profillemez.
docs_privacy_visibility_h = 4. Başkalarının görebildikleri
docs_privacy_visibility_p = Profilin, gönderilerin, yanıtların, yeniden paylaşımların, beğenilerin, takipçilerin ve takip ettiklerin, bu sunucuda oturum açmış herkese ve federasyondaki sunuculara açıktır. Biri bir gönderinin veya profilin bağlantısını paylaştığında Discord, Slack veya iMessage gibi uygulamalar herkese açık gönderiyi, görsellerini ve herkese açık profil ayrıntılarını gösteren bir önizleme alabilir. İçerik uyarısı olan veya hassas işaretli gönderiler metin ve görseller olmadan önizlenir. Yer işaretleri, engeller, sessize almalar, şikâyetler ve ayarlar özeldir. “Beni önerilerde göster” seçeneğini kapatırsan insanlar seni yine de tam kullanıcı adınla bulabilir.
docs_privacy_federation_h = 5. Federasyon
docs_privacy_federation_p = Başka bir tuita sunucusundaki birini takip ettiğinde, ona yanıt verdiğinde, ondan bahsettiğinde, gönderisini beğendiğinde, yeniden paylaştığında veya ona mesaj attığında gerekli verileri o sunucuya göndeririz: herkese açık profilin, gönderi veya etkileşim ve doğrudan mesajlar için mesaj metni. Diğer sunuculara giden doğrudan mesajlar uçtan uca şifreli değildir. Takipçilerinin sunucuları herkese açık gönderilerini ve profil güncellemelerini alır. Bu sunucular bağımsız işletilir ve verileri kendi politikalarına göre saklar. Bir gönderiyi veya hesabını sildiğinde federasyondaki sunuculardan kopyalarını silmelerini isteriz; ancak bunu yapacaklarını garanti edemeyiz.
docs_privacy_cookies_h = 6. Çerezler ve yerel depolama
docs_privacy_cookies_p = Kesinlikle gerekli, HttpOnly türünde tek bir oturum çerezi kullanırız. Tema, renk, yazı boyutu ve hareket tercihlerin tarayıcının yerel depolamasında tutulur ve bize asla gönderilmez.
docs_privacy_retention_h = 7. Verileri ne kadar saklıyoruz
docs_privacy_retention_1 = İçerik ve hesap verileri sen silene kadar saklanır.
docs_privacy_retention_2 = Silinen gönderiler hemen herkese açık görünümden kaldırılır; metinleri ve görselleri temizlenir.
docs_privacy_retention_3 = Hesabını sildiğinde gönderilerini, beğenilerini, takiplerini, mesajlarını ve oturumlarını kaldırır, hesap kaydını anonimleştiririz.
docs_privacy_retention_4 = Oturumlar 30 gün sonra veya çıkış yaptığında hemen sona erer.
docs_privacy_retention_5 = Moderasyon şikâyetleri ve yönetim denetim günlüğü, hizmeti güvende tutmak için gerektiği süre boyunca saklanır.
docs_privacy_rights_h = 8. Hakların
docs_privacy_rights_p = Yaşadığın yere göre (örneğin KVKK, GDPR veya CCPA kapsamında) verilerine erişme, düzeltme, dışa aktarma veya silme, ayrıca işlemeye itiraz etme veya kısıtlama hakkına sahip olabilirsin. Bunların çoğunu Ayarlar → Hesap bölümünden kendin yapabilirsin: profilini düzenle, verilerini indir ve hesabını sil. Başka bir konu için şu adrese başvur:
docs_privacy_rights_end = Yerel veri koruma otoritene şikâyette de bulunabilirsin.
docs_privacy_security_h = 9. Güvenlik
docs_privacy_security_p = Şifreler scrypt ile özetlenir. Oturum belirteçleri yalnızca özet olarak saklanır. Federasyondaki sunucular arasındaki istekler Ed25519 anahtarlarıyla imzalanır. Hiçbir sistem tamamen güvenli değildir; bu yüzden lütfen benzersiz bir şifre kullan.
docs_privacy_children_h = 10. Çocuklar
docs_privacy_children_p = Hizmet, Şartlarda belirtilen asgari yaşın altındaki çocuklar için değildir. Bir çocuğun hesabı olduğunu düşünüyorsan şikâyet et.
docs_privacy_changes_h = 11. Değişiklikler
docs_privacy_changes_p = Bu politikadaki değişiklikleri buraya koyar ve yukarıdaki tarihi güncelleriz. Önemli değişiklikler uygulama içinde duyurulur.
docs_privacy_contact_h = 12. İletişim
docs_privacy_contact_lead = Gizlilikle ilgili sorular veya talepler için:
profile_posts =
    { $count ->
        [one] { $compact } gönderi
       *[other] { $compact } gönderi
    }
profile_edit = Profili düzenle
profile_more = Daha fazla
profile_copy_link = Profil bağlantısını kopyala
profile_mute = { $handle } kişisini sessize al
profile_unmute = { $handle } kişisinin sesini aç
profile_block = { $handle } kişisini engelle
profile_unblock = { $handle } kişisinin engelini kaldır
profile_unblock_button = Engeli kaldır
profile_suspended = Bu hesap askıya alındı. Yalnızca ekip görebilir.
profile_muted = Sessize alındı
profile_org = Kuruluş
profile_role_admin = Yönetici
profile_role_moderator = Moderatör
profile_joined = Katılım: { $date }
profile_following_label = Takip edilenler
profile_followers =
    { $count ->
        [one] Takipçi
       *[other] Takipçi
    }
profile_counts = Takip sayıları
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] { $a } takip ediyor
            }
        [2]
            { $others ->
               *[other] { $a } ve { $b } takip ediyor
            }
       *[other]
            { $others ->
                [one] { $a }, { $b } ve takip ettiğin { $others } kişi daha takip ediyor
               *[other] { $a }, { $b } ve takip ettiğin { $others } kişi daha takip ediyor
            }
    }
profile_banner = Profil kapak görseli
profile_tabs = Profil bölümleri
profile_tab_posts = Gönderiler
profile_tab_replies = Yanıtlar
profile_tab_media = Medya
profile_tab_likes = Beğeniler
profile_posts_label = Gönderiler
profile_empty_posts_self_title = Henüz gönderi paylaşmadın
profile_empty_posts_self_body = Gönderilerin ve yeniden paylaşımların burada görünecek.
profile_empty_posts_title = { $handle } henüz paylaşmadı
profile_empty_posts_body = Paylaştığında gönderileri burada görünecek.
profile_empty_replies_title = Henüz yanıt yok
profile_empty_replies_body = Yanıtlar burada görünecek.
profile_empty_media_title = Henüz fotoğraf yok
profile_empty_media_body = Görselli gönderiler burada görünecek.
profile_empty_likes_title = Henüz beğeni yok
profile_empty_likes_body = Beğendiğin gönderiler burada görünecek.
profile_blocked_title = { $handle } kişisini engelledin
profile_blocked_body = Gönderilerini görmezsin; seni takip edemez ve sana mesaj atamaz.
profile_hidden_title = Bu gönderileri göremezsin
profile_hidden_body = { $handle } profilini kimlerin görebileceğini sınırlamış.
follows_tabs = Takip listeleri
follows_tab_followers = Takipçiler
follows_tab_following = Takip edilenler
follows_label_followers = Takipçiler
follows_label_following = Takip edilenler
follows_empty_followers_title = Henüz takipçi yok
follows_empty_followers_body = Birileri bu hesabı takip ettiğinde burada görünecek.
follows_empty_following_title = Kimseyi takip etmiyor
follows_empty_following_body = Takip ettiği hesaplar burada görünecek.
thread_title = Gönderi
thread_doc_title = { $name }: “{ $text }”
thread_views =
    { $count ->
        [one] Görüntülenme
       *[other] Görüntülenme
    }
thread_reposts =
    { $count ->
        [one] Yeniden paylaşım
       *[other] Yeniden paylaşım
    }
thread_quotes =
    { $count ->
        [one] Alıntı
       *[other] Alıntı
    }
thread_likes =
    { $count ->
        [one] Beğeni
       *[other] Beğeni
    }
thread_stats = Gönderi istatistikleri
thread_replies = Yanıtlar
thread_hidden = Gizli yanıtlar ({ $count })
thread_hidden_note = Yazar bu yanıtları gizledi.
thread_list_likes = Beğenenler
thread_list_reposts = Yeniden paylaşanlar
thread_list_quotes = Alıntılar
thread_list_empty = Henüz kimse yok
tag_empty_title = { $tag } etiketli gönderi yok
tag_empty_body = İlk kullanan sen ol.
tag_posts_label = { $tag } etiketli gönderiler
bookmarks_title = Yer işaretleri
bookmarks_label = Yer işaretli gönderiler
bookmarks_empty_title = Gönderileri sonra için kaydet
bookmarks_empty_body = Bir gönderiyi burada tutmak için yer işareti simgesine dokun. Yer işaretlerini yalnızca sen görürsün.
settings_account = Hesabın
settings_account_desc = Profil ve şifre
settings_profile = Profili düzenle
settings_profile_desc = Ad, biyografi, avatar şekli ve mürekkep
settings_privacy = Gizlilik ve güvenlik
settings_privacy_desc = Mesajlar, keşfedilebilirlik, sessize alınan kelimeler, engellenen ve sessize alınan hesaplar
settings_notifications = Bildirimler
settings_notifications_desc = Neler için bildirim alacağını seç
settings_appearance = Görünüm
settings_appearance_desc = Tema, mürekkep, yazı boyutu, kontrast ve hareket
settings_language = Dil
settings_language_desc = Arayüz dili
settings_language_note = Bu tarayıcıda ve oturum açtıysan hesabında kaydedilir.
settings_about = Bu sunucu hakkında
settings_about_desc = Kurallar, şartlar, gizlilik, sürüm ve klavye kısayolları
settings_logout = Çıkış yap
settings_account_profile = Profil
settings_account_joined = Katılım: { $date }
settings_account_edit = Profili düzenle
settings_password_title = Şifreyi değiştir
settings_password_current = Mevcut şifre
settings_password_new = Yeni şifre
settings_password_hint = En az 8 karakter. Diğer cihazlarda oturum kapatılacak.
settings_password_confirm = Yeni şifreyi onayla
settings_password_submit = Şifreyi güncelle
settings_password_mismatch = Yeni şifreler eşleşmiyor.
settings_password_updated = Şifre güncellendi. Diğer oturumlar kapatıldı.
settings_sessions_title = Cihazlar ve oturumlar
settings_sessions_desc = Tanımadığın oturumları kapat, ardından şifreni değiştir.
settings_device_on = { $os } üzerinde { $browser }
settings_device_unknown = Bilinmeyen cihaz
settings_session_current = Bu cihaz
settings_session_last_active = Son etkinlik: { $time }
settings_session_sign_out = Çıkış yap
settings_session_sign_out_device = { $device } cihazından çıkış yap
settings_session_signed_out = Oturum kapatıldı
settings_sessions_revoke_all = Diğer tüm cihazlardan çıkış yap
settings_sessions_revoke_title = Diğer cihazlardan çıkış yapılsın mı?
settings_sessions_revoke_body = Bu cihaz dışındaki tüm cihazlarda oturum kapatılacak.
settings_sessions_revoke_confirm = Diğerlerini kapat
settings_sessions_revoked =
    { $count ->
        [one] { $count } oturum kapatıldı
       *[other] { $count } oturum kapatıldı
    }
settings_profile_preview = Önizleme
settings_profile_preview_alt = Avatar önizlemesi
settings_profile_uploads = Avatar ve kapak görseli yükleme özelliği daha sonra gelecek. O zamana kadar senin için oluşturulan avatar ve kapak görseli kullanılır.
settings_profile_details = Ayrıntılar
settings_profile_name = Görünen ad
settings_profile_bio = Biyografi
settings_profile_location = Konum
settings_profile_website = Web sitesi
settings_profile_shape = Avatar şekli
settings_profile_ink = Profil mürekkebi
settings_profile_save = Profili kaydet
settings_profile_saved = Profil güncellendi
settings_shape_square = Kare
settings_shape_circle = Daire
settings_shape_squircle = Yuvarlak kare
settings_shape_octagon = Sekizgen
settings_ink_bone = Kemik
settings_ink_phosphor = Fosfor
settings_ink_amber = Kehribar
settings_ink_cyan = Camgöbeği
settings_ink_rose = Gül
settings_dm_title = Doğrudan mesajlar
settings_dm_desc = Seninle kimler sohbet başlatabilir
settings_dm_label = Sana kimler mesaj atabilir
settings_dm_everyone = Herkes
settings_dm_following = Takip ettiklerim
settings_dm_none = Hiç kimse
settings_dm_saved = Mesaj ayarları kaydedildi
settings_discover_title = Keşfedilebilirlik
settings_discoverable = Beni önerilerde ve aramada göster
settings_discoverable_hint = İnsanlar seni yine de tam kullanıcı adınla bulabilir
settings_show_client = Hangi cihazdan paylaştığımı göster
settings_show_client_hint = Yeni gönderilerinin altına cihazı ekler
settings_words_title = Sessize alınan kelimeler
settings_words_desc = Bu kelimeleri, ifadeleri veya #etiketleri içeren gönderileri zaman tünellerinden, aramadan ve yanıtlardan gizle. Kendi gönderilerin asla gizlenmez.
settings_words_label = Sessize alınacak kelime
settings_words_placeholder = Kelime, ifade veya #etiket ekle
settings_words_add = Sessize al
settings_words_remove = { $word } kelimesini sessizden çıkar
settings_words_none = Sessize alınan kelime yok.
settings_words_limit = En fazla { $count } kelimeyi sessize alabilirsin.
settings_blocked_title = Engellenen hesaplar
settings_blocked_desc = Seni takip edemez, sana mesaj atamaz, gönderilerini göremez veya onlarla etkileşime giremez. Kendilerine haber verilmez.
settings_blocked_empty = Kimseyi engellemedin.
settings_muted_title = Sessize alınan hesaplar
settings_muted_desc = Gönderilerini ve bildirimlerini görmezsin. Yine de seni takip edip mesaj atabilirler.
settings_muted_empty = Kimseyi sessize almadın.
settings_unblock = Engeli kaldır
settings_unmute = Sesi aç
settings_unblock_user = { $handle } kişisinin engelini kaldır
settings_unmute_user = { $handle } kişisinin sesini aç
settings_notif_title = Bana şunları bildir
settings_notif_desc = Sessize alınan ve engellenen hesaplar sana asla bildirim göndermez.
settings_notif_like = Beğeniler
settings_notif_like_hint = Biri gönderini beğendiğinde
settings_notif_repost = Yeniden paylaşımlar
settings_notif_repost_hint = Biri gönderini yeniden paylaştığında
settings_notif_reply = Yanıtlar
settings_notif_reply_hint = Biri sana yanıt verdiğinde
settings_notif_mention = Bahsetmeler
settings_notif_mention_hint = Biri senden bahsettiğinde
settings_notif_quote = Alıntılar
settings_notif_quote_hint = Biri gönderini alıntıladığında
settings_notif_follow = Yeni takipçiler
settings_notif_follow_hint = Biri seni takip ettiğinde
settings_desktop_title = Masaüstü bildirimleri
settings_desktop_desc = Yalnızca bu sekme arka plandayken sistem bildirimi olarak gösterilir.
settings_desktop_state = İzin: { $state }
settings_desktop_allowed = İzin verildi
settings_desktop_blocked = Engellendi
settings_desktop_not_asked = Sorulmadı
settings_desktop_enable = Etkinleştir
settings_desktop_blocked_toast = Bu site için bildirimler tarayıcı ayarlarında engellenmiş.
settings_desktop_unsupported = Bu tarayıcı masaüstü bildirimlerini desteklemiyor.
settings_theme_title = Tema
settings_appearance_note = Bu cihazda kaydedilir ve hemen uygulanır.
settings_theme_mode = Tema
settings_theme_system = Sistem
settings_theme_light = Açık
settings_theme_dark = Koyu
settings_theme_ink = Mürekkep
settings_theme_contrast = Kontrast
settings_contrast_normal = Normal
settings_contrast_more = Yüksek
settings_reading_title = Metin ve hareket
settings_theme_text_size = Yazı boyutu
settings_theme_motion = Hareket
settings_motion_system = Sistem
settings_motion_reduce = Azalt
settings_preview_title = Önizleme
settings_preview_text = Gönderiler ayarlarınla böyle görünecek.
settings_about_domain = Alan adı
settings_about_version = Sürüm
settings_about_contact = İletişim
settings_about_terms = Hizmet Şartları
settings_about_privacy = Gizlilik Politikası
settings_about_source = Kaynak kodu
settings_about_rules = Kurallar
explore_search_label = Gönderileri, kişileri ve etiketleri ara
explore_search_placeholder = Gönderi, kişi, #etiket ara
explore_clear = Aramayı temizle
explore_tips = Arama ipuçları
explore_tips_title = Arama operatörleri
explore_op_from = Bir hesabın gönderileri
explore_op_to = Bir hesaba gelen yanıtlar veya ondan bahsedenler
explore_op_media = Görselli gönderiler (veya has:poll, has:links)
explore_op_since = Bir tarihte veya sonrasında paylaşılan
explore_op_until = Bir tarihte veya öncesinde paylaşılan
explore_op_phrase = Bu sırada geçen kelimeler
explore_op_exclude = Bir kelime içeren gönderileri hariç tut
explore_results = Arama sonuçları
explore_tab_posts = Gönderiler
explore_tab_people = Kişiler
explore_tab_media = Medya
explore_people = Kişiler
explore_posts = Gönderiler
explore_recent = Son aramalar
explore_recent_clear = Temizle
explore_trending = Gündemde
explore_trend_posts =
    { $count ->
        [one] { $compact } gönderi
       *[other] { $compact } gönderi
    }
explore_trends_empty = Henüz gündem yok. Bir #etiket kullan!
explore_suggestions = Kimi takip etmeli
explore_suggestions_empty = Herkesi takip ediyorsun. Etkileyici.
explore_no_posts_title = “{ $q }” için gönderi yok
explore_no_posts_body = Başka bir kelime veya etiket dene.
explore_no_people_title = “{ $q }” için kişi yok
explore_no_people_body = Bir ad, kullanıcı adı ya da başka bir sunucudaki biri için @kullanıcı@sunucu dene.
explore_no_media_title = “{ $q }” için görsel yok
explore_no_media_body = Eşleşen görselli gönderiler burada görünecek.
notif_filter = Bildirim filtresi
notif_tab_all = Tümü
notif_tab_mentions = Bahsetmeler
notif_empty_all_title = Henüz bildirim yok
notif_empty_all_body = Beğeniler, yanıtlar, yeniden paylaşımlar ve yeni takipçiler burada görünecek.
notif_empty_mentions_title = Henüz bahsetme yok
notif_empty_mentions_body = Gönderilerine gelen yanıtlar, bahsetmeler ve alıntılar burada görünür.
notif_who_two = { $first } ve { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } ve { $count } kişi daha
       *[other] { $first }, { $second } ve { $count } kişi daha
    }
notif_verb_like =
    { $count ->
        [one] gönderini beğendi
       *[other] gönderini beğendi
    }
notif_verb_repost =
    { $count ->
        [one] gönderini yeniden paylaştı
       *[other] gönderini yeniden paylaştı
    }
notif_verb_follow =
    { $count ->
        [one] seni takip etmeye başladı
       *[other] seni takip etmeye başladı
    }
notif_verb_follow_request =
    { $count ->
        [one] sana takip isteği gönderdi
       *[other] sana takip isteği gönderdi
    }
notif_verb_reply =
    { $count ->
        [one] sana yanıt verdi
       *[other] sana yanıt verdi
    }
notif_verb_mention =
    { $count ->
        [one] senden bahsetti
       *[other] senden bahsetti
    }
notif_verb_quote =
    { $count ->
        [one] gönderini alıntıladı
       *[other] gönderini alıntıladı
    }
notif_verb_poll =
    { $count ->
        [one] oy verdiğin bir anketi sonlandırdı
       *[other] oy verdiğin bir anketi sonlandırdı
    }
notif_verb_affiliation =
    { $count ->
        [one] seni bağlı hesap olmaya davet etti
       *[other] seni bağlı hesap olmaya davet etti
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Zamanla
composer_post_all = Hepsini gönder
composer_err_type = Bu dosya tuita'nın kullanabileceği bir görsel veya video değil (PNG, JPEG, GIF, WebP, MP4, WebM veya MOV).
composer_err_image_size = Görsel çok büyük (en fazla 10 MB).
composer_err_video_size = Videolar en fazla { $mb } MB olabilir.
composer_err_max_images = En fazla 4 görsel ekleyebilirsin.
composer_err_exclusive = Bir gönderide video, görsel veya anket olabilir; ama yalnızca biri.
composer_extra_placeholder = Başka bir gönderi ekle
composer_extra_label = Dizi gönderisi { $n }
composer_extra_remove = { $n }. dizi gönderisini kaldır
composer_schedule_chip = { $time } tarihinde paylaşılacak
composer_schedule_clear = Zamanlamayı kaldır
composer_add_images = Görsel ekle
composer_add_video = Video ekle
composer_add_poll = Anket ekle
composer_add_post = Başka bir gönderi ekle
composer_scheduled = { $time } için zamanlandı
composer_thread_sent = { $count } gönderilik dizi paylaşıldı
composer_remove_video = Videoyu kaldır
composer_remove_image = Görseli kaldır
composer_uploading = Yükleniyor
composer_uploading_video = Video yükleniyor… %{ $percent }
composer_alt_video = Bu videoyu tanımla
composer_alt_image = Bu görseli tanımla
composer_alt_label = { $index }. ekin açıklaması (alternatif metin)
composer_emoji = Emoji ekle
poll_length_hours = { $n } sa
poll_length_days = { $n } g
poll_choice = Seçenek { $n }
poll_choice_optional = Seçenek { $n } (isteğe bağlı)
poll_remove_choice = { $n }. seçeneği kaldır
poll_add_choice = Seçenek ekle
poll_length = Anket süresi
poll_multiple_label = Birden çok seçime izin ver
poll_remove = Anketi kaldır
schedule_title = Gönderiyi zamanla
schedule_hour = Bir saat sonra
schedule_evening = Bu akşam
schedule_tomorrow = Yarın sabah
schedule_week = Bir hafta sonra
schedule_window = Şu andan itibaren bir dakika ile 90 gün arasında bir zaman seç.
schedule_custom = Tarih ve saat seç
schedule_set = Ayarla
schedule_clear = Zamanlamayı kaldır
schedule_view = Zamanlanmış gönderileri gör
dropzone_drop = Eklemek için bırak
dropzone_reject = Bu dosya kullanılamaz
nav_messages = Mesajlar
nav_lists = Listeler
nav_scheduled = Zamanlanmış
notify_message = @{ $handle } sana mesaj gönderdi
shortcut_go_messages = Mesajlar
shortcut_go_lists = Listeler
cropper_stage = Kırpma alanı. Görseli taşımak için sürükle. Hafifçe oynatmak için ok tuşlarını, yakınlaştırmak için artı ve eksi tuşlarını kullan.
cropper_hint = Konumlandırmak için sürükle. Yakınlaştırmak için kıstır, kaydır veya kaydırıcıyı kullan.
cropper_zoom = Yakınlaştırma
cropper_zoom_in = Yakınlaştır
cropper_zoom_out = Uzaklaştır
cropper_rotate = Döndür
cropper_reset = Sıfırla
cropper_apply = Uygula
image_field_avatar = Profil fotoğrafı
image_field_banner = Kapak görseli
image_field_edit_avatar = Profil fotoğrafını düzenle
image_field_edit_banner = Kapak görselini düzenle
image_field_change = Değiştir
image_field_remove = Kaldır
image_field_hint = 10 MB'a kadar PNG, JPEG, GIF veya WebP. GIF hareketsiz bir görsele dönüşür.
image_field_not_image = Bu dosya tuita'nın kullanabileceği bir görsel değil (PNG, JPEG, GIF veya WebP).
image_field_too_large = Görsel çok büyük (en fazla 10 MB).
image_field_uploading = Yükleniyor
image_field_saved = Görsel kaydedildi.
image_field_removed = Görsel kaldırıldı.
edit_title = Gönderiyi düzenle
edit_note = Paylaştıktan sonra 30 dakika boyunca düzenleyebilirsin. İnsanlar bir gönderinin düzenlendiğini ve önceki sürümlerini görebilir.
edit_save = Kaydet
edit_saved = Gönderi güncellendi
edit_edited = Düzenlendi
edit_history_title = Düzenleme geçmişi
edit_history_current = Güncel sürüm
edit_history_earlier = Önceki sürüm
edit_history_original = Orijinal
report_post_title = Gönderiyi şikâyet et
report_user_title = { $handle } kişisini şikâyet et
report_intro = Sorun ne? Bu sunucunun moderatörleri şikâyetini inceleyecek. { $handle } kendisini kimin şikâyet ettiğini görmeyecek.
report_reason = Neden
report_note = Başka bir şey? (isteğe bağlı)
report_send = Şikâyeti gönder
report_sent = Teşekkürler. Şikâyetin moderatörlere gönderildi.
report_duplicate = Bunu zaten şikâyet ettin. Moderatörler göz atacak.
report_spam = Spam
report_spam_body = İstenmeyen reklam, dolandırıcılık, tekrarlayan veya yanıltıcı içerik
report_harassment = Taciz
report_harassment_body = Birini hedef almak, zorbalık veya tehdit
report_hate = Nefret
report_hate_body = İnsanlara kimlikleri nedeniyle saldırıyor
report_violence = Şiddet veya kendine zarar
report_violence_body = Tehdit, şiddeti övme, kendine zarar vermeye teşvik
report_misinformation = Yanlış bilgi
report_misinformation_body = Zarar verebilecek yanlış iddialar
report_impersonation = Kimliğe bürünme
report_impersonation_body = Başka biri gibi davranma
report_other = Başka bir şey
report_other_body = Kuralları başka bir şekilde ihlal ediyor
post_edit = Gönderiyi düzenle
post_edit_history = Düzenleme geçmişi
post_report = Gönderiyi şikâyet et
post_hide_reply = Yanıtı gizle
post_unhide_reply = Yanıtı göster
post_share_menu = Paylaş…
post_reply_hidden = Gizli yanıt
post_reply_hidden_toast = Yanıt gizlendi
post_reply_unhidden_toast = Yanıt yeniden gösteriliyor
sensitive_label = Hassas medya
sensitive_show = Göster
sensitive_hide = Hassas medyayı gizle
media_label =
    { $count ->
        [one] { $count } ek
       *[other] { $count } ek
    }
media_video = Video
media_open = { $count } görselden { $index }. görseli aç
lightbox_title = Medya görüntüleyici
lightbox_prev = Önceki
lightbox_next = Sonraki
poll_vote = Oy ver
poll_your_vote = Verdiğin oy
poll_votes =
    { $count ->
        [one] { $compact } oy
       *[other] { $compact } oy
    }
poll_final = Nihai sonuçlar
poll_multiple = çoktan seçmeli
poll_left_minutes = { $n } dk kaldı
poll_left_hours = { $n } sa kaldı
poll_left_days = { $n } g kaldı
pull_refresh = Yenilemek için aşağı çek
messages_title = Mesajlar
messages_new = Yeni mesaj
messages_label = Sohbetler
messages_empty_title = Bir sohbet başlat
messages_empty_body = Gönderdiğin ve aldığın mesajlar burada görünür.
messages_you_prefix = Sen: { $text }
messages_unread = { $count } okunmamış
messages_new_search = Kişi ara
messages_new_results = Kişiler
messages_new_none = Kimse bulunamadı.
chat_load_older = Önceki mesajları yükle
chat_intro = Sohbetinin başlangıcı burası.
chat_label = { $name } ile mesajlar
chat_delete = Sil
chat_seen = Görüldü
chat_delete_title = Bu mesaj silinsin mi?
chat_delete_body = İkiniz için de kaybolur.
chat_doc_title = { $name } ile sohbet
chat_error_title = Bu sohbet açılamadı
chat_placeholder = Mesaj yaz
chat_input_label = Mesaj
chat_send = Gönder
lists_title = Listeler
lists_new = Yeni liste
lists_edit = Listeyi düzenle
lists_label = Listelerin
lists_empty_title = Henüz liste yok
lists_empty_body = Gönderilerini ayrı okumak için kişileri listelerde grupla.
lists_members =
    { $count ->
        [one] { $compact } üye
       *[other] { $compact } üye
    }
lists_private = Özel
lists_name = Ad
lists_description = Açıklama
lists_private_switch = Özel liste
lists_save = Kaydet
lists_picker_title = { $handle } kişisini listelere ekle
lists_picker_none = Henüz listen yok.
lists_delete = Listeyi sil
lists_delete_title = Bu liste silinsin mi?
lists_delete_body = “{ $name }” kaldırılacak. İçindeki kişiler etkilenmez.
lists_by = { $handle } kişisinin listesi
lists_tab_posts = Gönderiler
lists_tab_members = Üyeler
lists_tabs = Liste bölümleri
lists_error_title = Bu liste açılamadı
lists_posts_label = Liste gönderileri
lists_posts_empty_title = Burada henüz bir şey yok
lists_posts_empty_body = Bu listedeki üyelerin gönderileri burada görünür.
lists_members_label = Liste üyeleri
lists_members_empty_title = Üye yok
lists_members_empty_body = Kişileri profillerinden ekle.
lists_remove = Kaldır
scheduled_title = Zamanlanmış gönderiler
scheduled_label = Zamanlanmış gönderiler
scheduled_status_pending = Zamanlandı
scheduled_status_published = Paylaşıldı
scheduled_status_failed = Başarısız
scheduled_cw = İçerik uyarısı: { $text }
scheduled_poll = { $count } seçenekli anket, { $hours } sa sürer
scheduled_change_time = Saati değiştir
scheduled_post_now = Şimdi gönder
scheduled_remove = Kaldır
scheduled_window_error = Şu andan itibaren bir dakika ile 90 gün arasında bir zaman seç.
scheduled_when = Paylaşım zamanı
scheduled_zone = Saat dilimi: { $zone }
scheduled_save = Kaydet
scheduled_posted = Paylaşıldı.
scheduled_cancel_title = Bu zamanlanmış gönderi iptal edilsin mi?
scheduled_cancel_body = Paylaşılmayacak.
scheduled_cancel_confirm = Gönderiyi iptal et
scheduled_empty_title = Zamanlanmış bir şey yok
scheduled_empty_body = Zamanladığın gönderiler vakti gelene kadar burada bekler.
profile_message = Mesaj
profile_add_to_lists = Listelere ekle
media_alt = ALT
profile_report = { $handle } kişisini şikâyet et
chart_show_table = { $title } verilerini tablo olarak göster
chart_col_day = Gün
chart_col_hour = Saat (UTC)
chart_empty = { $title }: henüz gösterilecek bir şey yok.
chart_bar_summary = { $title }: toplam { $total }, en yüksek { $label } ({ $peak }).
chart_spark_summary = { $title }: toplam { $total }, { $label } günü { $last }.
chart_heat_summary = { $title }: toplam { $total }, en yoğun { $label } günü saat { $hour } ({ $peak }).
chart_heat_less = Az
chart_heat_more = Daha fazla
announce_region = Duyurular
announce_dismiss = Duyuruyu kapat
announce_more = Devamını oku
announce_tone_info = Bilgi
announce_tone_success = İyi haber
announce_tone_warning = Uyarı
admin_stat_people = Kişiler
admin_stat_people_sub = bu hafta { $count } etkin
admin_stat_posts_today = Gönderiler, 24 sa
admin_stat_posts_total = toplam { $count }
admin_stat_reports = Açık şikâyetler
admin_stat_reports_open = İnceleme gerekiyor
admin_stat_reports_clear = Her şey yolunda
admin_stat_signups_today = Bugünkü kayıtlar
admin_stat_signups_sub = 14 günde { $count }
admin_stat_suspended = Askıya alınan
admin_stat_suspended_sub = hesap
admin_chart_posts = Günlük gönderiler
admin_chart_signups = Günlük kayıtlar
admin_chart_reports = Günlük şikâyetler
admin_chart_heat = Haftanın gününe ve saate göre etkinlik
nav_admin = Yönetim
admin_title = Yönetim
admin_sections = Yönetim bölümleri
admin_tab_overview = Genel bakış
admin_tab_reports = Şikâyetler
admin_tab_users = Kişiler
admin_tab_instance = Sunucu
admin_tab_federation = Federasyon
admin_tab_announcements = Duyurular
admin_tab_invites = Davetler
admin_tab_filter = İçerik filtresi
admin_tab_audit = Denetim günlüğü
admin_denied_title = Yalnızca ekip
admin_denied_body = Bu alan, bu sunucunun yöneticileri ve moderatörleri içindir.
admin_denied_home = Ana sayfaya git
admin_unknown = bilinmiyor
admin_tag_active = Etkin
admin_tag_suspended = Askıya alındı
admin_tag_deleted = Silindi
admin_tag_org = Kuruluş
admin_audit_system = sistem
admin_audit_empty = Henüz kayıt yok
admin_report_status = Şikâyet durumu
admin_report_open = Açık
admin_report_resolved = Çözüldü
admin_reports_empty_open = Açık şikâyet yok
admin_reports_empty_open_body = İnsanlar gönderileri veya hesapları şikâyet ettiğinde burada görünür.
admin_reports_empty_resolved = Henüz çözülen yok
admin_report_by = şikâyet eden: { $handle }
admin_report_same_target = { $count } açık şikâyet
admin_report_media =
    { $count ->
        [one] { $count } ek
       *[other] { $count } ek
    }
admin_report_poll = Anket içeriyor
admin_report_dismiss = Kapat
admin_report_delete_post = Gönderiyi sil
admin_report_suspend = Hesabı askıya al
admin_report_delete_suspend = Sil ve askıya al
admin_report_confirm_title = İşlem yapılsın mı?
admin_report_confirm_body = { $action }. Bu, denetim günlüğüne kaydedilir.
admin_report_confirm = Onayla
admin_report_resolution = { $handle } tarafından { $action } · { $time }
admin_res_dismiss = Kapatıldı
admin_res_delete_post = Gönderi silindi
admin_res_suspend_user = Hesap askıya alındı
admin_res_delete_and_suspend = Gönderi silindi ve hesap askıya alındı
admin_users_search = Kişi ara
admin_users_search_placeholder = Ad veya kullanıcı adı
admin_users_show = Göster
admin_users_empty = Kimse bulunamadı
admin_filter_local = Yerel
admin_filter_remote = Uzak
admin_filter_staff = Ekip
admin_filter_verified = Doğrulanmış
admin_filter_suspended = Askıya alınan
admin_filter_all = Herkes
admin_user_details = { $handle } hesabının ayrıntıları
admin_user_menu = { $handle } kişisini yönet
admin_user_posts =
    { $count ->
        [one] { $compact } gönderi
       *[other] { $compact } gönderi
    }
admin_user_followers =
    { $count ->
        [one] { $compact } takipçi
       *[other] { $compact } takipçi
    }
admin_user_joined = katılım: { $time }
admin_user_active = son etkinlik: { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } açık şikâyet
       *[other] { $compact } açık şikâyet
    }
admin_act_verify = Doğrula
admin_act_unverify = Doğrulamayı kaldır
admin_act_org = Kuruluş yap
admin_act_unorg = Kuruluş durumunu kaldır
admin_act_role_user = Ekip rolünü kaldır
admin_act_role_moderator = Moderatör yap
admin_act_role_admin = Yönetici yap
admin_act_suspend = Askıya al
admin_act_unsuspend = Askıyı kaldır
admin_act_purge = Tüm gönderileri sil
admin_act_rename = Kullanıcı adını değiştir
admin_act_reset_name = Görünen adı sıfırla
admin_act_clear_bio = Biyografiyi temizle
admin_act_delete = Hesabı sil
admin_done_verified = { $handle } doğrulandı
admin_done_unverified = Doğrulama kaldırıldı
admin_done_org = { $handle } artık bir kuruluş
admin_done_unorg = Kuruluş durumu kaldırıldı
admin_done_role = Rol güncellendi
admin_done_suspended = { $handle } askıya alındı
admin_done_unsuspended = { $handle } askıdan çıkarıldı
admin_done_purged =
    { $count ->
        [one] { $count } gönderi silindi
       *[other] { $count } gönderi silindi
    }
admin_done_name_reset = Görünen ad sıfırlandı
admin_done_bio_cleared = Biyografi temizlendi
admin_done_deleted = { $handle } silindi
admin_done_renamed = Yeni kullanıcı adı: { $handle }
admin_role_phrase_user = normal kullanıcı
admin_role_phrase_moderator = moderatör
admin_role_phrase_admin = yönetici
admin_role_title = { $handle } kişisinin rolü değiştirilsin mi?
admin_role_body = Yeni rolü: { $role }.
admin_role_confirm = Rolü değiştir
admin_unorg_title = { $handle } kişisinin kuruluş durumu kaldırılsın mı?
admin_unorg_body = Tüm bağlı hesap ilişkileri sona erer.
admin_unorg_confirm = Kaldır
admin_suspend_title = { $handle } askıya alınsın mı?
admin_suspend_body = Oturumları kapatılır ve askıyı kaldırana kadar içerikleri gizli kalır.
admin_suspend_confirm = Askıya al
admin_purge_title = { $handle } kişisinin tüm gönderileri silinsin mi?
admin_purge_body = Bu işlem geri alınamaz.
admin_purge_confirm = Hepsini sil
admin_reset_name_title = { $handle } kişisinin görünen adı sıfırlansın mı?
admin_reset_name_body = Görünen adın yerine kullanıcı adı kullanılacak.
admin_reset_name_confirm = Sıfırla
admin_delete_title = { $handle } silinsin mi?
admin_delete_body = Gönderileri, takipleri ve mesajları kaldırılır, hesap anonimleştirilir. Bu işlem geri alınamaz.
admin_delete_confirm = Hesabı sil
admin_rename_title = { $handle } kişisinin kullanıcı adını değiştir
admin_rename_note = Eski kullanıcı adına giden bağlantılar çalışmaz.
admin_rename_handle = Yeni kullanıcı adı
admin_rename_reset_name = Görünen adı da sıfırla
admin_rename_save = Kullanıcı adını değiştir
admin_detail_title = Hesap ayrıntıları
admin_detail_invite = Davet { $code }
admin_detail_posts = Gönderiler
admin_detail_followers = Takipçiler
admin_detail_following = Takip edilenler
admin_detail_open_reports = Açık şikâyetler
admin_detail_sessions = Oturumlar
admin_detail_passkeys = Geçiş anahtarları
admin_detail_tokens = API belirteçleri
admin_detail_joined = Katılım: { $time }
admin_detail_recent_posts = Son gönderiler
admin_detail_no_posts = Gönderi yok.
admin_detail_reports = Bu hesapla ilgili şikâyetler
admin_detail_no_reports = Şikâyet yok.
admin_detail_history = Moderasyon geçmişi
admin_detail_nothing = Henüz yok.
admin_instance_title = Sunucu
admin_instance_name = Sunucu adı
admin_instance_description = Açıklama
admin_instance_contact = İletişim e-postası
admin_instance_contact_hint = Şartlar ve Gizlilik sayfalarında gösterilir
admin_instance_min_age = Asgari yaş
admin_instance_rules = Kurallar
admin_instance_rules_hint = Her satıra bir kural
admin_instance_preview = Şartları önizle
admin_instance_save = Kaydet
admin_instance_saved = Sunucu ayarları kaydedildi
admin_instance_template_note = Hizmet Şartları ve Gizlilik Politikası, bu değerleri kullanan şablonlardır. Sunucuyu herkese açmadan önce bulunduğun ülkenin mevzuatına göre incelettir.
admin_signups_title = Kayıtlar
admin_signups_desc = Bu sunucuda kimlerin hesap oluşturabileceği
admin_signups_open = Açık
admin_signups_open_hint = Herkes hesap oluşturabilir.
admin_signups_invite = Yalnızca davetle
admin_signups_invite_hint = İnsanların Davetler sekmesinden bir davet koduna ihtiyacı var.
admin_signups_closed = Kapalı
admin_signups_closed_hint = Kimse kaydolamaz.
admin_filter_title = İçerik filtresi
admin_filter_desc = Aşağıya kelime ekleyene kadar hiçbir şey engellenmez. Engellenen kelimeler; kullanıcı adlarına, görünen adlara, konumlara, biyografilere, gönderilere ve liste adlarına uygulanır; gizlenmiş yazımlar da dahildir. “admin” gibi ayrılmış kullanıcı adları kaydedilemez.
admin_filter_blocked = Engellenen kelimeler
admin_filter_blocked_hint = Her satıra bir tane, kelimelerin içinde de eşleşir
admin_filter_allowed = İzin verilen kelimeler
admin_filter_allowed_hint = Engellenen kelimelere istisnalar, her satıra bir tane
admin_filter_reserved = Ek ayrılmış kullanıcı adları
admin_filter_reserved_hint = Her satıra bir tane
admin_filter_save = Filtreyi kaydet
admin_filter_saved = Filtre kaydedildi
admin_filter_field_handle = kullanıcı adı
admin_filter_field_name = ad
admin_filter_field_location = konum
admin_filter_field_bio = biyografi
admin_scan_title = Hesapları tara
admin_scan_desc = Kullanıcı adı, adı veya biyografisi filtreyi ihlal eden hesapları bul.
admin_scan_run = Şimdi tara
admin_scan_clean = Filtreyi ihlal eden hesap yok.
admin_scan_found =
    { $count ->
        [one] { $count } hesap dikkat gerektiriyor
       *[other] { $count } hesap dikkat gerektiriyor
    }
admin_scan_reason = { $field } şunu içeriyor: { $reason }
admin_ann_title = Yeni duyuru
admin_ann_desc = Süresi dolana veya sen bitirene kadar herkesin ekranının üstünde görünen bir şerit. İnsanlar kapatabilir.
admin_ann_text = Mesaj
admin_ann_link = Bağlantı
admin_ann_link_hint = İsteğe bağlı: https://… veya /yol
admin_ann_tone = Stil
admin_ann_tone_info = Bilgi
admin_ann_tone_success = İyi haber
admin_ann_tone_warning = Uyarı
admin_ann_duration = Süre
admin_ann_days =
    { $count ->
        [one] { $count } gün
       *[other] { $count } gün
    }
admin_ann_week = 1 hafta
admin_ann_until_ended = Bitirene kadar
admin_ann_publish = Yayınla
admin_ann_published = Duyuru yayınlandı
admin_ann_list = Şimdiye kadarki duyurular
admin_ann_none = Henüz duyuru yok.
admin_ann_end = Bitir
admin_ann_ended = bitirildi
admin_ann_expired = süresi doldu
admin_ann_inactive = Gösterilmiyor
admin_ann_until = { $date } tarihine kadar
admin_inv_title = Davet kodları
admin_inv_desc = Kayıtlar yalnızca davetle olduğunda belirli kişilerin katılmasına izin ver.
admin_inv_note = Not (kimin için)
admin_inv_uses = Kullanım
admin_inv_use_count =
    { $count ->
        [one] { $count } kullanım
       *[other] { $count } kullanım
    }
admin_inv_expiry = Bitiş
admin_inv_week = 1 hafta sonra
admin_inv_days =
    { $count ->
        [one] { $count } gün sonra
       *[other] { $count } gün sonra
    }
admin_inv_never = Hiçbir zaman
admin_inv_create = Kod oluştur
admin_inv_created = { $code } kodu oluşturuldu
admin_inv_created_copied = { $code } kodu oluşturuldu ve kopyalandı
admin_inv_list = Kodlar
admin_inv_none = Henüz davet kodu yok.
admin_inv_state_active = Etkin
admin_inv_state_revoked = İptal edildi
admin_inv_state_used = Tükendi
admin_inv_state_expired = Süresi doldu
admin_inv_used = { $uses }/{ $max } kullanıldı
admin_inv_expires = bitiş: { $date }
admin_inv_copy = { $code } kodunu kopyala
admin_inv_copied = Kod kopyalandı
admin_inv_copy_failed = Kopyalanamadı. Kod: { $code }.
admin_inv_revoke = İptal et
admin_inv_revoke_title = Bu davet iptal edilsin mi?
admin_inv_revoke_body = Bu davetle zaten katılmış kişiler hesaplarını korur.
settings_devices = Cihazlar ve oturumlar
settings_devices_desc = Nerede giriş yaptığını gör ve uzaktan çıkış yap
settings_passkeys = Geçiş anahtarları
settings_passkeys_desc = Parmak izin, yüzün, ekran kilidin veya bir güvenlik anahtarıyla giriş yap
settings_affiliation = Kuruluş bağlantısı
settings_affiliation_desc = Parçası olduğun kuruluşlar veya kuruluşunun hesapları
settings_bots = Botlar ve API
settings_bots_desc = API belirteçleri ve “Otomatik” etiketi
settings_saved = Kaydedildi
passkey_title = Geçiş anahtarların
passkey_desc = Şifre yerine parmak izin, yüzün, ekran kilidin veya güvenlik anahtarınla giriş yap. Geçiş anahtarları oltalanamaz ve sunucumuzda gizli hiçbir şey saklanmaz.
passkey_none = Henüz geçiş anahtarı yok.
passkey_add = Geçiş anahtarı ekle
passkey_added = “{ $name }” geçiş anahtarı eklendi
passkey_default_name = Geçiş anahtarı
passkey_added_ago = eklenme: { $time }
passkey_used_ago = son kullanım: { $time }
passkey_never_used = hiç kullanılmadı
passkey_rename = Yeniden adlandır
passkey_rename_label = { $name } geçiş anahtarını yeniden adlandır
passkey_rename_title = Geçiş anahtarını yeniden adlandır
passkey_name = Ad
passkey_rename_save = Kaydet
passkey_remove = Kaldır
passkey_remove_label = { $name } geçiş anahtarını kaldır
passkey_remove_title = “{ $name }” kaldırılsın mı?
passkey_remove_body = Bu geçiş anahtarıyla artık giriş yapamazsın. Onu cihazından veya şifre yöneticinden de silmek isteyebilirsin.
passkey_removed = Geçiş anahtarı kaldırıldı
passkey_unsupported = Bu tarayıcı bu adreste geçiş anahtarlarını kullanamıyor.
passkey_needs_domain = Geçiş anahtarları için bu sitenin IP adresi değil, bir alan adı olması gerekir.
passkey_already_here = Bu cihazda bu hesap için zaten bir geçiş anahtarı var.
token_scope_read = Okuma
token_scope_write = Paylaşım ve etkileşim
token_scope_messages = Doğrudan mesajlar
token_scope_read_hint = Zaman tünelleri, bildirimler, profiller, arama ve gerçek zamanlı akış
token_scope_write_hint = Paylaşma, silme, beğeniler, yeniden paylaşımlar, yer işaretleri, takipler ve yüklemeler
token_scope_messages_hint = Doğrudan mesajları okuma ve gönderme
token_title = API belirteçleri
token_desc = Belirteçler, betiklerin ve botların hesabını API üzerinden kullanmasını sağlar. Onlara şifre gibi davran: belirteci olan herkes, kapsamları dahilinde senin adına işlem yapabilir.
token_none = Henüz belirteç yok.
token_name = Belirteç adı
token_name_hint = Örneğin: hava durumu botu
token_scopes = Kapsamlar
token_create = Belirteç oluştur
token_used_ago = son kullanım: { $time }
token_never_used = hiç kullanılmadı
token_revoke = İptal et
token_revoke_label = { $name } belirtecini iptal et
token_revoke_title = “{ $name }” iptal edilsin mi?
token_revoke_body = Onu kullanan her şey çalışmayı durdurur.
token_revoked = Belirteç iptal edildi
token_secret_title = Yeni belirtecin
token_secret_warning = Şimdi kopyala. Güvenliğin için tekrar gösterilmeyecek.
token_secret_label = Yeni API belirteci
token_copy = Kopyala
token_copied = Belirteç kopyalandı
token_copy_failed = Kopyalanamadı. Belirteci seçip elle kopyala.
token_done = Tamam
bot_title = Otomatik hesap
bot_desc = Adın yanında “Otomatik” etiketi gösterir; böylece insanlar bir botun paylaştığını bilir.
bot_switch = Bu otomatik bir hesap
bot_owner = İşleten
bot_owner_hint = Arkasındaki kişinin veya kuruluşun kullanıcı adı. İsteğe bağlı.
bot_owner_save = Kaydet
bot_docs = Geliştirici belgeleri
bot_docs_desc = Uç noktalar, gerçek zamanlı olaylar ve örnek botlar
aff_yours_title = Kuruluş bağlantın
aff_yours_desc = Bir kuruluş, rozetini adının yanında göstermen için seni davet edebilir. Kabul edene kadar hiçbir şey değişmez ve istediğin zaman ayrılabilirsin.
aff_none = Bir kuruluşa bağlı değilsin.
aff_invites_title = Davetler
aff_accept = Kabul et
aff_decline = Reddet
aff_leave = Ayrıl
aff_leave_title = Bu kuruluştan ayrılınsın mı?
aff_leave_body = Rozeti profilinden kalkar.
aff_remove_title = { $handle } kaldırılsın mı?
aff_remove_body = Rozetin onun profilinden kalkar.
aff_remove = Kaldır
aff_remove_label = { $handle } kişisini kaldır
aff_ended = Bağlantı sona erdi
aff_declined = Davet reddedildi
aff_accepted = Bağlantı eklendi
aff_invited = Davet gönderildi
aff_org_title = Bağlı hesapların
aff_org_desc = İnsanları, adlarının yanında rozetini göstermeye davet et. Kabul etmeleri gerekir.
aff_handle = Kullanıcı adı
aff_label = Etiket
aff_label_hint = İsteğe bağlı, örneğin “Mühendis”
aff_invite = Davet gönder
aff_members_none = Henüz bağlı hesap yok.
aff_pending = Davet edildi
aff_not_org = Bir kuruluş musun? Bu sunucunun bir yöneticisinden hesabını işaretlemesini iste.
auth_passkey = Geçiş anahtarıyla giriş yap
auth_or = veya
auth_invite = Davet kodu
auth_invite_hint = Bu sunucu yeni kişileri yalnızca davetle alır.
auth_invite_required = Sana verilen davet kodunu gir.
landing_developers = Geliştiriciler
landing_invite = Kayıt yalnızca davetle. Kodun var mı? Hesabını oluştur.
dev_title = Geliştiriciler
dev_hero = Botlar ve API
dev_hero_sub = { $name } için botlar, betikler ve entegrasyonlar geliştir
dev_intro_h = { $name } için geliştir
dev_intro_p = tuita uygulamalarının yaptığı her şey küçük bir JSON API üzerinden geçer; botlar ve betikler de bunu kullanabilir. { $name } üzerinde bir hesaba ve kişisel bir API belirtecine ihtiyacın var. Uygulama kaydı yok, OAuth karmaşası yok.
dev_intro_1 = Botun için bir hesap oluştur veya kendi hesabını kullan.
dev_intro_2 = Ayarlar, Botlar ve API bölümünde “Bu otomatik bir hesap” seçeneğini aç; böylece insanlar “Otomatik” etiketini görür. Botu kimin işlettiğini de belirt.
dev_intro_3 = İhtiyacın olan kapsamlarla bir belirteç oluştur ve gizli tut.
dev_auth_h = Kimlik doğrulama
dev_auth_p = Belirteci Authorization başlığında gönder. Çok parçalı (multipart) yüklemeler dışında her istek ve yanıt JSON'dur (UTF-8).
dev_auth_prefix = Belirteç, tuita_ ve ardından 43 harf, rakam, tire ve alt çizgiden oluşur. Yalnızca özeti saklanır; bu yüzden kaybolan belirteç tekrar gösterilemez: yenisini oluştur. Bir belirteç şu kapsamlardan bir veya birkaçına sahip olur:
dev_auth_refused = Belirteç; şifreleri, oturumları, belirteçleri veya geçiş anahtarlarını değiştiremez, hesabı dışa aktaramaz veya silemez ve yönetim API'sini kullanamaz. Bunlar web sitesinde gerçek bir girişle yapılır ve 403 yanıtı döner.
dev_auth_errors = Hatalar, bir error mesajı ve 4xx durumuyla JSON olarak döner. 401 belirtecin yanlış veya iptal edilmiş olduğu, 403 kapsamın eksik olduğu veya yolun reddedildiği anlamına gelir; 429 yanıtı Retry-After başlığıyla gelir.
dev_post_h = İlk gönderin
dev_post_p = Önce POST /api/v1/media ile (multipart) görselleri ve videoyu yükle, sonra mediaIds içinde kimlikleriyle ekle. Yanıt için replyTo, alıntı için quoteOf ayarlanır.
dev_endpoints_h = Uç noktalar
dev_endpoints_intro = Yollardaki süslü parantezli kısımlar yer tutucudur. Listeler, önceki sayfanın sonraki değeriyle ayarlanan ?before= ile sayfalanır; limit 1 ile 40 arasındadır. Kapsam sütunu, belirtecin ihtiyaç duyduğu kapsamdır.
dev_col_method = Yöntem
dev_col_path = Yol
dev_col_scope = Kapsam
dev_col_what = Ne yapar
dev_ep_me = Belirtecin ait olduğu hesap
dev_ep_patch_me = O hesabın profilini ve ayarlarını değiştirir
dev_ep_bot = “Otomatik” etiketini açar veya kapatır ve botu işleten kişiyi belirtir
dev_ep_following = Takip ettiğin kişilerin ana zaman tüneli
dev_ep_foryou = Sana özel zaman tüneli; ?cursor= ile sayfalanır
dev_ep_notifications = Bildirimler (yanıtlar, bahsetmeler ve alıntılar için ?filter=mentions)
dev_ep_post = Dizisi ve yanıtlarıyla bir gönderi
dev_ep_user = Bir profil (diğer sunuculardaki hesaplar için kullanıcı@alanadı)
dev_ep_user_posts = Birinin gönderileri (?tab=replies, media veya likes; media ile ayrıca ?kind=photo veya video)
dev_ep_search = Gönderilerde, kişilerde veya medyada ara; from: ve has:media gibi operatörler kullanılabilir
dev_ep_trends = Son haftada en çok kullanılan etiketler
dev_ep_stream = Gerçek zamanlı olaylar (Server-Sent Events)
dev_ep_create = Gönderi yaz
dev_ep_thread = 2 ila 25 gönderiyi dizi olarak yaz
dev_ep_delete = Gönderilerinden birini sil
dev_ep_like = Bir gönderiyi beğen (geri almak için DELETE). /repost ve /bookmark için de geçerli
dev_ep_follow = Birini takip et (takibi bırakmak için DELETE)
dev_ep_media = Görsel veya videoyu multipart olarak yükle; yanıt, eklenecek kimliği içerir
dev_ep_schedule = Bir gönderiyi daha sonrası için zamanla
dev_ep_report = Bir gönderiyi veya hesabı moderatörlere şikâyet et
dev_ep_conversations = Doğrudan mesaj sohbetlerin
dev_ep_message = Doğrudan mesaj gönder; gövde, text alanı olan bir JSON'dur
dev_realtime_h = Gerçek zamanlı olaylar
dev_realtime_p = GET /api/v1/stream bir Server-Sent Events akışıdır. Her beğeni, yeniden paylaşım, takip, yanıt, bahsetme veya alıntı için bir notification olayı, doğrudan mesajlar için bir message olayı ve okunmamış sayılarıyla rozetler gönderir. Her 25 saniyede bir yorum satırı gönderir ve istemcilerden 5 saniye sonra yeniden denemelerini ister; bağlantı koparsa yeniden bağlan.
dev_example_h = Örnek: bahsetmelere yanıt veren bir bot
dev_example_node = Node 22 veya daha yenisi, paket yok:
dev_example_python = Aynısı Python'da, yalnızca requests ile:
dev_limits_h = Sınırlar
dev_limit_rate = Her belirteç 10 dakikada 600 istek yapabilir. Sınır aşılırsa sunucu Retry-After başlığıyla 429 yanıtı verir.
dev_limit_tokens = Bir hesapta en fazla 20 belirteç olabilir ve saatte en fazla 20 belirteç oluşturulabilir.
dev_limit_posting = Paylaşma, takip etme, şikâyet etme ve diğer işlemlerin hesap başına kendi sınırları vardır.
dev_rules_h = Botlar için kurallar
dev_rule_1 = Otomatik etiketini aç ve botu kimin işlettiğini belirt.
dev_rule_2 = Yalnızca isteyen kişilere yanıt ver veya mesaj at: bottan bahsetmiş, onu takip etmiş veya ona önce mesaj atmış olsunlar. İstenmeyen bahsetme veya DM gönderme.
dev_rule_3 = Ölçülü davran: yukarıdaki sınırların çok altında kal.
dev_rule_4 = Sunucunun Şartlarına ve kurallarına uy. Moderatörler botları da diğer hesaplar gibi askıya alabilir.
admin_act_log_admin_bootstrap = İlk yönetici kuruldu
admin_act_log_announcement_create = Duyuru yayınlandı
admin_act_log_announcement_end = Duyuru bitirildi
admin_act_log_instance_settings = Sunucu ayarları değiştirildi
admin_act_log_invite_create = Davet oluşturuldu
admin_act_log_invite_revoke = Davet iptal edildi
admin_act_log_post_delete = Gönderi silindi
admin_act_log_report_dismiss = Şikâyet kapatıldı
admin_act_log_user_delete = Hesap silindi
admin_act_log_user_org = Kuruluş olarak işaretlendi
admin_act_log_user_unorg = Kuruluş durumu kaldırıldı
admin_act_log_user_purge_posts = Tüm gönderiler silindi
admin_act_log_user_rename = Hesap yeniden adlandırıldı
admin_act_log_user_role = Rol değiştirildi
admin_act_log_user_suspend = Hesap askıya alındı
admin_act_log_user_unsuspend = Hesabın askısı kaldırıldı
admin_act_log_user_unverify = Doğrulama kaldırıldı
admin_act_log_user_verify = Hesap doğrulandı
admin_act_log_domain_block = Sunucu engellendi
admin_act_log_domain_unblock = Sunucunun engeli kaldırıldı
admin_act_log_federation_retry = Teslimatların yeniden denenmesi istendi
admin_fed_stat_domains = Bilinen sunucular
admin_fed_stat_domains_sub = bu sunucunun gördüğü
admin_fed_stat_accounts = Uzak hesaplar
admin_fed_stat_accounts_sub = başka yerlerden bilinen
admin_fed_stat_posts = Uzak gönderiler
admin_fed_stat_posts_sub = yerelde saklanan
admin_fed_stat_queue = Kuyrukta
admin_fed_stat_queue_sub = bekleyen teslimat
admin_fed_queue_title = Teslimat kuyruğu
admin_fed_queue_desc = Diğer sunuculara ulaşmayı bekleyen imzalı etkinlikler. Başarısız olanlar artan aralıklarla yeniden denenir ve 12 denemeden sonra kuyruktan çıkarılır.
admin_fed_queue_empty = Bekleyen bir şey yok. Tüm teslimatlar gönderildi.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } teslimat bekliyor
       *[other] { $compact } teslimat bekliyor
    }
admin_fed_queue_due = { $count } tanesinin zamanı geldi
admin_fed_queue_oldest = En eski kuyruğa alınan: { $date }
admin_fed_retry = Şimdi yeniden dene
admin_fed_retry_done =
    { $count ->
        [one] { $compact } teslimat şimdi yeniden denenecek.
       *[other] { $compact } teslimat şimdi yeniden denenecek.
    }
admin_fed_failing = Başarısız sunucular
admin_fed_failing_none = Başarısız olan sunucu yok.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } teslimat
       *[other] { $compact } teslimat
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } deneme
       *[other] { $compact } deneme
    }
admin_fed_failing_next = sonraki deneme: { $date }
admin_fed_failing_error = Son hata
admin_fed_failing_no_error = Kayıtlı hata yok
admin_fed_actor_title = Sunucu aktörü
admin_fed_actor_desc = Diğer sunucular, bu sunucunun imzaladığı istekleri bu anahtarla doğrular.
admin_fed_actor_id = Aktör
admin_fed_actor_key = Anahtar kimliği
admin_fed_actor_fingerprint = SHA-256 parmak izi
admin_fed_actor_copy = Parmak izini kopyala
admin_fed_actor_copied = Parmak izi kopyalandı
admin_fed_actor_copy_failed = Kopyalanamadı. Parmak izi: { $fingerprint }
admin_fed_blocks_title = Engellenen sunucular
admin_fed_blocks_desc = Susturma, bir sunucunun hesaplarını herkese açık zaman tünellerinden ve aramadan gizler. Askıya alma, onları her yerden gizler, gönderdiklerini yok sayar ve onlara teslimatları durdurur.
admin_fed_domain = Alan adı
admin_fed_domain_hint = example.social veya üzerindeki herhangi bir sayfaya bağlantı
admin_fed_severity = Yaptırım
admin_fed_silence = Sustur
admin_fed_suspend = Askıya al
admin_fed_reason = Neden
admin_fed_reason_hint = İsteğe bağlı, yalnızca ekibe gösterilir
admin_fed_block = Sunucuyu engelle
admin_fed_blocked = { $domain } engellendi
admin_fed_none = Engellenen sunucu yok.
admin_fed_list = Engeller
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } bilinen hesap
       *[other] { $compact } bilinen hesap
    }
admin_fed_row_no_reason = Neden belirtilmedi
admin_fed_row_changed = { $domain } için yaptırım: { $severity }
admin_fed_remove = Kaldır
admin_fed_remove_label = { $domain } engelini kaldır
admin_fed_remove_title = { $domain } engeli kaldırılsın mı?
admin_fed_remove_body = Hesapları ve gönderileri geri gelir; bu sunucuya teslimatlar yeniden başlar.
admin_fed_removed = { $domain } artık engelli değil
remote_domain_title = { $domain } üzerindeki hesap
remote_view_original = { $domain } üzerinde gör
visibility_unlisted = Listelenmeyen
visibility_unlisted_hint = Herkese açık zaman tünellerinde gösterilmez
visibility_followers = Yalnızca takipçiler
visibility_followers_hint = Bunu yalnızca takipçiler görebilir
visibility_direct = Doğrudan
visibility_direct_hint = Bunu yalnızca bahsedilen kişiler görebilir
follow_requested = İstek gönderildi
follow_cancel_request = Takip isteğini iptal et
follow_requested_toast = { $handle } kişisine takip isteği gönderildi
follow_cancelled_toast = Takip isteği iptal edildi
profile_locked = Takipçileri onaylıyor
profile_locked_hint = Takip istekleri bu hesabın onayını gerektirir.
profile_original = Profili { $domain } üzerinde gör
profile_remote_note = { $domain } sunucusundan alınan bir kopya. Sayılar ve gönderiler bu sunucunun gördükleridir.
remote_lookup_loading = { $handle } kişisi { $domain } üzerinde aranıyor…
remote_lookup_notfound_title = Böyle bir hesap yok
remote_lookup_notfound_body = { $handle } yok ya da { $domain } bunu paylaşmıyor.
remote_lookup_failed_title = { $handle } yüklenemedi
explore_lookup = { $target } aranıyor…
explore_lookup_empty_body = Bu hesap veya gönderi bulunamadı. Diğer sunucu çevrimdışı olabilir ya da bunu paylaşmıyor olabilir.
language_label = Dil
oauth_title = Bir uygulamayı yetkilendir
oauth_loading = İstek denetleniyor…
oauth_wants = Bu uygulama { $handle } hesabını kullanmak istiyor.
oauth_name_warning = Herkes istediği uygulama adını seçebilir. Yalnızca güvendiğin uygulamaları onayla.
oauth_website = Web sitesi
oauth_allow = Şunları yapabilecek
oauth_scope_read = Profilini, gönderilerini, takiplerini, listelerini, zaman tünellerini, bildirimlerini ve uçtan uca şifreli olmayan doğrudan mesajlarını okuma.
oauth_scope_write = Senin adına paylaşma, düzenleme ve silme; beğenme, takip etme, engelleme, sessize alma; profilini ve ayarlarını değiştirme.
oauth_scope_messages = Doğrudan mesajlarını okuma ve gönderme.
oauth_push_note = Anlık bildirimler desteklenmez. İsteyen uygulamalar da bunları alamaz.
oauth_redirect = Yanıt verdikten sonra şuraya yönlendirilirsin
oauth_redirect_oob = Uygulama yönlendirme almaz. Uygulamaya yapıştırman için burada bir kod gösterilir.
oauth_approve = Yetkilendir
oauth_deny = Reddet
oauth_switch = Başka bir hesap kullan
oauth_sent_approved = Yetkilendirildi. { $host } adresine geri gönderiliyorsun. Hiçbir şey olmazsa bu sekmeyi kapat.
oauth_sent_denied = Reddedildi. { $host } adresine geri gönderiliyorsun. Hiçbir şey olmazsa bu sekmeyi kapat.
oauth_denied = Reddedildi. Uygulamayla hiçbir şey paylaşılmadı.
oauth_code_title = Yetkilendirme kodu
oauth_code_body = Bu kodu uygulamaya yapıştır. Bir kez çalışır ve 10 dakika içinde sona erer.
oauth_code_copy = Kodu kopyala
oauth_code_copied = Kod kopyalandı
oauth_code_copy_failed = Kopyalanamadı. Kodu seçip elle kopyala.
oauth_error_title = Bu istek yetkilendirilemez
oauth_error_note = Uygulamaya hiçbir şey gönderilmedi.
oauth_error_invalid_request = İstek hatalı ya da uygulamanın kaydetmediği bir adrese dönmeyi istiyor. Uygulamaya geri dön ve tekrar dene.
oauth_error_invalid_client = Bu uygulama burada kayıtlı değil.
oauth_error_invalid_scope = Uygulama, tuita'nın sunmadığı bir erişim veya kaydolduğundan fazlasını istiyor.
oauth_error_unsupported_response_type = Uygulama, tuita'nın desteklemediği bir giriş türü istiyor.
oauth_error_rate = Çok fazla isteğe yanıt verdin. Birkaç dakika bekleyip tekrar dene.
oauth_error_unreachable = tuita'ya ulaşılamadı. Bağlantını denetleyip tekrar dene.
oauth_home = tuita'ya git
share_send_chat = Sohbetle gönder
share_download = Medyayı indir
share_download_n = { $number }. medyayı indir
share_chat_title = Gönderiyi sohbetle gönder
share_chat_sent = { $handle } kişisine gönderildi
hover_card_label = { $name } profil kartı
profile_affiliation_role = ({ $label })
profile_media_filter = Medya türü
profile_media_all = Tüm medya
profile_media_photos = Fotoğraflar
profile_media_videos = Videolar
profile_empty_videos_title = Henüz video yok
profile_empty_videos_body = Videolu gönderiler burada görünecek.
account_add_existing = Mevcut bir hesap ekle
rail_label = Arama ve öneriler
rail_search_submit = Ara
rail_show_more = Daha fazla göster
rail_footer = Bağlantılar
chat_new = Yeni sohbet
chat_search = Sohbetlerde ara
chat_search_none = Aramanızla eşleşen sohbet yok.
chat_start_title = Sohbet başlat
chat_start_body = Listeden bir sohbet seçin veya yenisini başlatın.
explore_sections = Keşfet bölümleri
explore_see_more = Daha fazla göster
history_title = Geçmiş
history_tabs = Geçmiş bölümleri
history_likes_label = Beğenilen gönderiler
video_play = Oynat
video_pause = Duraklat
video_mute = Sessize al
video_unmute = Sesi aç
video_volume = Ses düzeyi
video_seek = Sar
video_position = { $total } içinde { $current }
video_speed = Oynatma hızı
video_fullscreen = Tam ekran
video_exit_fullscreen = Tam ekrandan çık
video_pip = Pencere içinde pencere
video_exit_pip = Pencere içinde pencereden çık
video_error = Bu video oynatılamıyor.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Paneli gizle
lightbox_show_panel = Paneli göster
lightbox_view_post = Gönderiyi görüntüle
lightbox_panel = Gönderi ve yanıtlar
emotepick_tabs = Emoji setleri
emotepick_tab_emoji = Emoji
emotepick_tab_custom = Özel
admin_emote_tab = Emotlar
admin_emote_title = Özel emotlar
admin_emote_desc = Emotlar tüm sunucuya aittir. Herkes bunları gönderilerde, adlarda ve mesajlarda :shortcode: biçiminde kullanabilir. Kısa kod yeniden adlandırılamaz; emoti silip yeniden yükleyin.
admin_emote_shortcode = Kısa kod
admin_emote_shortcode_hint = 2 ila 32 harf, rakam veya alt çizgi. :shortcode: olarak gösterilir.
admin_emote_invalid = Yalnızca harf, rakam ve alt çizgi kullanın, 2 ila 32 karakter.
admin_emote_category = Kategori
admin_emote_category_hint = İsteğe bağlı. Emotları seçicide gruplar.
admin_emote_file = Emot görseli
admin_emote_file_hint = PNG, JPEG, WebP veya GIF, en fazla 256 KB. Buraya bir dosya bırakın ya da seçin.
admin_emote_file_type = Bu dosya PNG, JPEG, WebP veya GIF görseli değil.
admin_emote_file_size = Görsel çok büyük (en fazla 256 KB).
admin_emote_choose_file = Görsel seç
admin_emote_change_file = Görseli değiştir
admin_emote_upload = Emotu yükle
admin_emote_upload_failed = Emot yüklenemedi. Tekrar deneyin.
admin_emote_uploaded = :{ $shortcode }: eklendi.
admin_emote_list = Bu sunucudaki emotlar
admin_emote_none = Henüz özel emot yok.
admin_emote_no_category = Kategori yok
admin_emote_category_for = :{ $shortcode }: kategorisi
admin_emote_saved = Kategori kaydedildi.
admin_emote_delete = :{ $shortcode }: sil
admin_emote_delete_title = Bu emot silinsin mi?
admin_emote_delete_body = :{ $shortcode }: artık hiçbir yerde görsel olarak gösterilmez, düz metin olarak görünür. Onu kullanan gönderiler metni korur.
admin_emote_delete_confirm = Emotu sil
admin_emote_deleted = Emot silindi.
admin_act_log_emoji_create = Emote eklendi
admin_act_log_emoji_update = Emote değiştirildi
admin_act_log_emoji_delete = Emote silindi
e2ee_settings = Şifreli mesajlar
e2ee_settings_desc = Şifre cümlesi, bu cihaz ve sıfırlama
e2ee_status_label = Durum
e2ee_status_none = Kapalı. Uçtan uca şifreli mesaj göndermek ve okumak için aç.
e2ee_status_locked = Bu cihazda kilitli. Mesajlarını burada okumak için şifre cümleni gir.
e2ee_status_ready = Açık ve bu cihazda kilidi açık.
e2ee_keys_error = Şifreleme anahtarların denetlenemedi.
e2ee_setup_title = Şifreli mesajları aç
e2ee_setup_body = Bu sunucudaki kişilere giden mesajlar tarayıcında şifrelenir. Sunucu yalnızca okuyamadığı metni saklar. Bir şifre cümlesi, mesajlarının kilidini başka bir cihazda açmanı sağlar.
e2ee_setup_warning = Şifre cümlen kurtarılamaz. Unutursan şifrelemeyi sıfırlayabilirsin ama önceki mesajların kaybolur.
e2ee_setup_submit = Aç
e2ee_setup_done = Şifreli mesajlar açık
e2ee_unlock_title = Şifreli mesajların kilidini aç
e2ee_unlock_body = Bu cihazda şifreli mesajları okumak ve göndermek için şifre cümleni gir.
e2ee_unlock_submit = Kilidi aç
e2ee_unlock_wrong = Bu şifre cümlesi yanlış.
e2ee_unlock_done = Kilit açıldı
e2ee_passphrase = Şifre cümlesi
e2ee_passphrase_confirm = Şifre cümlesini onayla
e2ee_passphrase_hint = En az 10 karakter.
e2ee_passphrase_short = En az 10 karakter kullan.
e2ee_passphrase_mismatch = Şifre cümleleri eşleşmiyor.
e2ee_failed = Bir şeyler ters gitti. Tekrar dene.
e2ee_unsupported = Bu tarayıcı şifreli mesajları desteklemiyor.
e2ee_forget = Bu cihazda unut
e2ee_forget_hint = Anahtarını ve güvendiğin kişileri bu tarayıcıdan kaldırır. Şifre cümlen bunların kilidini yeniden açar.
e2ee_forget_done = Bu cihazda unutuldu
e2ee_reset = Şifrelemeyi sıfırla
e2ee_reset_hint = Yeni bir anahtar oluşturur. Daha önce gönderdiğin ve aldığın mesajlar okunamaz hale gelir.
e2ee_reset_title = Şifreleme sıfırlansın mı?
e2ee_reset_body = Önceki mesajların hem senin hem de yazdığın kişiler için okunamaz hale gelir ve geri getirilemez. Kişilerinden yeni anahtarına güvenmeleri istenir. Yeni bir şifre cümlesi seç.
e2ee_reset_submit = Sıfırla
e2ee_reset_done = Şifreleme sıfırlandı
e2ee_note_sealed = Uçtan uca şifreli
e2ee_note_remote = Uçtan uca şifreli değil: bu kişi başka bir sunucuda.
e2ee_note_plain = Uçtan uca şifreli değil.
e2ee_plain_mark = Şifresiz
e2ee_preview = Şifreli mesaj
e2ee_line_sealed = Şifreli mesaj
e2ee_line_own_key = Artık sahip olmadığın bir anahtara gönderildi
e2ee_line_peer_key = Henüz güvenmediğin bir anahtarla gönderildi
e2ee_line_unreadable = Açılamadı
e2ee_line_unsupported = Bu tarayıcı şifreli mesajları açamıyor
e2ee_block_peer_off = { $handle } henüz şifreli mesajları açmadı.
e2ee_block_self_off = Önce Ayarlar’dan şifreli mesajları aç.
e2ee_block_unlock = Önce Ayarlar’dan şifreli mesajların kilidini aç.
e2ee_block_downgrade = { $handle } daha önce şifreli mesajlar kullandı. Bu sohbet şifresiz mesajlara geri dönmeyecek.
e2ee_block_changed = Yazabilmek için { $handle } kişisinin yeni anahtarına güven.
e2ee_block_error = Şifreleme anahtarların denetlenemedi.
e2ee_changed_title = { $name } kişisinin şifreleme anahtarı değişti
e2ee_changed_body = Şifrelemeyi sıfırlamış ya da yeni bir cihaza geçmiş olabilir; ya da araya biri girmiş olabilir. Güvenmeden önce yeni güvenlik numarasını onunla karşılaştır. Bunu yapana kadar mesaj gönderemezsin.
e2ee_changed_old = Daha önce güvendiğin güvenlik numarası
e2ee_changed_new = Yeni güvenlik numarası
e2ee_trust = Yeni anahtara güven
e2ee_safety_open = Güvenlik numarası
e2ee_safety_title = Güvenlik numarası
e2ee_safety_body = Bu numarayı { $name } ile yüz yüze veya bir görüşmede karşılaştır, bu sohbette değil. İki ekranda da aynıysa aranızda kimse yok.

## 0031
composer_upload_failed = Yükleme başarısız oldu. Tekrar dene.
composer_send_failed = tuita'ya ulaşılamadı. Bağlantını kontrol edip tekrar dene.
composer_open_failed = Gönderi alanı açılamadı. Tekrar dene.
