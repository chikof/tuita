app_name = tuita
loading = Chargement
close = Fermer
dismiss = Ignorer
notifications = Notifications
avatar_more = { $count } de plus
char_counter_left =
    { $count ->
        [one] { $count } caractère restant
       *[other] { $count } caractères restants
    }
char_counter_over =
    { $count ->
        [one] { $count } caractère au-dessus de la limite
       *[other] { $count } caractères au-dessus de la limite
    }
not_found_title = Rien ici
not_found_body = Cette page n’existe pas, ou elle a été déplacée.
not_found_back = Retour
tagline = Chaque post, deux encres.
error_title = Une erreur s’est produite
cancel = Annuler
retry = Réessayer
back = Retour
view = Voir
follow = Suivre
following = Abonnements
follows_you = Vous suit
verified = Compte vérifié
bot_account = Compte automatisé
bot_label = Bot
link_copied = Lien copié dans le presse-papiers
text_copied = Texte copié
time_now = maintenant
time_minutes = { $n } min
time_hours = { $n } h
time_days = { $n } j
nav_main = Principal
nav_home = Accueil
nav_explore = Explorer
nav_notifications = Notifications
nav_bookmarks = Signets
nav_profile = Profil
nav_settings = Paramètres
nav_compose = Publier
nav_menu = Menu
badge_unread = Non lus : { $count }
account_menu = Comptes
account_switch = Changer de compte
account_switch_to = Passer à { $handle }
account_add = Ajouter un compte
account_logout = Déconnecter { $handle }
account_logout_all = Déconnecter tous les comptes
home_feed = Fil
home_for_you = Pour vous
home_following = Abonnements
home_empty_title = Rien pour l’instant
home_empty_body = Soyez le premier à publier.
home_following_empty_title = Votre cercle est calme
home_following_empty_body = Suivez des personnes pour remplir ce fil avec leurs posts et repartages.
home_find_people = Trouver des personnes
new_posts =
    { $count ->
        [one] Afficher { $count } nouveau post
       *[other] Afficher { $count } nouveaux posts
    }
new_posts_many = Afficher 99+ nouveaux posts
composer_title = Nouveau post
composer_reply_title = Répondre
composer_quote_title = Citer le post
composer_placeholder = Quoi de neuf ?
composer_reply_placeholder = Publiez votre réponse
composer_text_label = Texte du post
composer_replying_to = En réponse à { $handle }
composer_cw_label = Avertissement de contenu
composer_cw_placeholder = Avertissement de contenu, p. ex. spoilers
composer_cw_toggle = Ajouter un avertissement de contenu
composer_sensitive_toggle = Marquer les médias comme sensibles
composer_options = Options du post
composer_policy_label = Qui peut répondre
composer_suggestions = Suggestions
composer_tag_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
composer_post = Publier
composer_reply = Répondre
policy_everyone = Tout le monde peut répondre
policy_following = Les personnes que vous suivez peuvent répondre
policy_mentioned = Seules les personnes mentionnées peuvent répondre
post_sent = Post envoyé
post_by = Post de { $name }
post_reposted_by = { $name } a repartagé
post_pinned = Post épinglé
post_replying_to = En réponse à { $handle }
post_replying_unavailable = En réponse à un post indisponible
post_unavailable = Ce post n’est pas disponible.
post_menu = Plus
post_copy_link = Copier le lien
post_copy_text = Copier le texte
post_pin = Épingler au profil
post_unpin = Désépingler du profil
post_pinned_toast = Épinglé à votre profil
post_unpinned = Désépinglé de votre profil
post_delete = Supprimer le post
post_delete_title = Supprimer le post ?
post_delete_body = Cette action est irréversible. Le post est retiré de votre profil, des fils des personnes qui vous suivent et de la recherche.
post_deleted = Post supprimé
post_view_profile = Voir { $handle }
post_mute = Masquer { $handle }
post_block = Bloquer { $handle }
post_reply_label = Répondre ({ $count })
post_reply_following_only = Seules les personnes suivies par l’auteur peuvent répondre.
post_reply_mentioned_only = Seules les personnes mentionnées dans ce post peuvent répondre.
post_repost_menu = Options de repartage
post_repost = Repartager
post_undo_repost = Annuler le repartage
post_quote = Citer
post_reposts_label = Repartages et citations ({ $count })
post_like_label = J’aime ({ $count })
post_bookmark_label = Ajouter aux signets
post_share = Partager ou copier le lien
cw_show = Afficher
cw_hide = Masquer
cw_label = Avertissement de contenu : { $reason }
muted_toast = { $handle } masqué
unmuted_toast = { $handle } n’est plus masqué
blocked_toast = { $handle } bloqué
unblocked_toast = { $handle } débloqué
block_title = Bloquer { $handle } ?
block_body = Cette personne ne pourra plus vous suivre et vous ne verrez plus ses posts. Elle n’en est pas informée.
block_confirm = Bloquer
notify_like = @{ $handle } a aimé votre post
notify_repost = @{ $handle } a repartagé votre post
notify_reply = @{ $handle } vous a répondu
notify_mention = @{ $handle } vous a mentionné
notify_quote = @{ $handle } a cité votre post
notify_follow = @{ $handle } vous suit désormais
notify_follow_request = @{ $handle } a demandé à vous suivre
notify_poll = Un sondage auquel vous avez voté est terminé
notify_affiliation = @{ $handle } vous a invité à vous affilier à son organisation
shortcuts_title = Raccourcis clavier
shortcut_group_navigation = Navigation
shortcut_group_actions = Actions
shortcut_or = /
shortcut_then = puis
shortcut_plus = +
shortcut_next_prev = Post suivant / précédent
shortcut_open = Ouvrir le post
shortcut_search = Rechercher
shortcut_go_home = Accueil
shortcut_go_explore = Explorer
shortcut_go_notifications = Notifications
shortcut_go_bookmarks = Signets
shortcut_go_profile = Votre profil
shortcut_go_settings = Paramètres
shortcut_new_post = Nouveau post
shortcut_like = J’aime
shortcut_reply = Répondre
shortcut_repost = Repartager ou citer
shortcut_bookmark = Ajouter aux signets
shortcut_send = Envoyer le post
shortcut_help = Afficher cette aide
auth_login_title = Se connecter
auth_add_title = Ajouter un autre compte
auth_add_sub = Connectez-vous pour passer d’un compte à l’autre (jusqu’à { $max }) sur cet appareil sans vous déconnecter.
auth_signup_title = Créer un compte
auth_handle = Identifiant
auth_password = Mot de passe
auth_display_name = Nom affiché
auth_handle_hint = 3 à 15 lettres, chiffres ou tirets bas
auth_password_hint = Au moins 8 caractères
auth_consent_pre = J’ai au moins { $minAge } ans et j’accepte les{" "}
auth_consent_terms = Conditions d’utilisation
auth_consent_and = {" "}et la{" "}
auth_consent_privacy = Politique de confidentialité
auth_consent_end = {"."}
auth_consent_required = Veuillez accepter les conditions d’utilisation et la politique de confidentialité.
auth_signin = Se connecter
auth_signin_add = Se connecter et changer
auth_create = Créer un compte
auth_to_signup = Pas encore de compte ? Créer un compte
auth_to_login = Déjà un compte ? Se connecter
auth_closed = Les inscriptions sont actuellement fermées sur cette instance.
landing_nav = Pied de page
landing_rules = Règles de l’instance
landing_stats = En ce moment
landing_people = Personnes
landing_posts = Posts
landing_clock = Votre heure locale
landing_trends = Tendances
landing_trend =
    { $count ->
        [one] { $count } post
       *[other] { $count } posts
    }
landing_join = Rejoindre { $name }
landing_join_sub = Créez un compte et publiez en deux encres.
landing_closed = Les inscriptions sont fermées pour le moment. Vous avez déjà un compte ? Connectez-vous.
landing_terms = Conditions
landing_privacy = Confidentialité
landing_source = Code source
landing_powered = Propulsé par tuita { $version }
docs_terms_title = Conditions d’utilisation
docs_privacy_title = Politique de confidentialité
docs_updated = Dernière mise à jour : { $date }
docs_date = 5 novembre 2026
docs_see_privacy = Voir aussi la politique de confidentialité.
docs_see_terms = Voir aussi les conditions d’utilisation.
docs_contact_fallback = les administrateurs de l’instance (par message privé ou par signalement)
docs_terms_agreement_h = 1. Accord
docs_terms_agreement_p = Les présentes conditions d’utilisation (« Conditions ») régissent votre usage de { $name }, l’instance tuita située à { $domain } (« le service », « nous »). En créant un compte ou en utilisant le service, vous acceptez ces Conditions et notre politique de confidentialité. Si vous n’êtes pas d’accord, n’utilisez pas le service.
docs_terms_eligibility_h = 2. Qui peut l’utiliser
docs_terms_eligibility_p = Vous devez avoir au moins { $minAge } ans, ou l’âge minimum de consentement numérique là où vous vivez s’il est plus élevé. Vous devez pouvoir conclure un accord contraignant et ne pas être interdit d’utiliser le service par la loi applicable.
docs_terms_account_h = 3. Votre compte
docs_terms_account_1 = Gardez votre mot de passe secret. Vous êtes responsable de l’activité sur votre compte.
docs_terms_account_2 = Votre identifiant ne doit pas usurper l’identité d’une autre personne ou organisation, ni induire en erreur sur qui vous êtes.
docs_terms_account_3 = Prévenez-nous rapidement si vous pensez que votre compte a été compromis. Vous pouvez déconnecter les autres sessions depuis Paramètres → Compte.
docs_terms_content_h = 4. Votre contenu
docs_terms_content_p1 = Vous restez propriétaire de ce que vous publiez. Vous nous accordez une licence mondiale, non exclusive et gratuite pour stocker, afficher, reproduire et diffuser votre contenu, uniquement pour faire fonctionner et améliorer le service. Cela inclut son envoi aux instances tuita fédérées (voir la section 6). Cette licence prend fin lorsque vous supprimez le contenu ou votre compte, sauf pour les copies déjà transmises à d’autres instances ou enregistrées par d’autres personnes.
docs_terms_content_p2 = Vous êtes responsable de ce que vous publiez. Ne publiez que du contenu que vous avez le droit de partager.
docs_terms_rules_h = 5. Règles de la communauté
docs_terms_rules_lead = En utilisant { $name }, vous vous engagez à ne pas enfreindre la loi ni ces règles :
docs_terms_rules_p = Vous vous engagez aussi à ne pas attaquer le service, par exemple en essayant de contourner les mesures de sécurité, en l’aspirant à grande échelle, en envoyant des logiciels malveillants ou en le surchargeant.
docs_terms_federation_h = 6. Fédération
docs_terms_federation_p = { $name } peut se connecter (« fédérer ») avec d’autres instances tuita indépendantes approuvées par les administrateurs. Lorsque vous interagissez avec des personnes d’une autre instance, votre profil public, vos posts publics et l’interaction (par exemple un abonnement, un j’aime, un repartage, une réponse ou un message privé) sont envoyés à cette instance. Les autres instances sont gérées par d’autres personnes selon leurs propres conditions. Nous ne pouvons pas contrôler ni supprimer un contenu dès lors qu’une autre instance l’a reçu, mais nous lui envoyons des demandes de suppression lorsque vous supprimez quelque chose.
docs_terms_moderation_h = 7. Modération et sanctions
docs_terms_moderation_p = Les modérateurs peuvent retirer du contenu, limiter des fonctionnalités ou suspendre des comptes qui enfreignent ces Conditions ou la loi. Si possible, nous vous en expliquerons la raison. Chacun peut signaler un post ou un compte. Les signalements sont transmis aux modérateurs de cette instance. Nous pouvons cesser de fédérer avec toute instance à tout moment.
docs_terms_termination_h = 8. Départ
docs_terms_termination_p = Vous pouvez exporter vos données ou supprimer votre compte à tout moment depuis Paramètres → Compte. Nous pouvons suspendre ou mettre fin à votre accès si vous enfreignez gravement ou à répétition ces Conditions. Nous pouvons aussi fermer le service, et nous essaierons de vous prévenir raisonnablement à l’avance.
docs_terms_disclaimers_h = 9. Exclusions de garantie et responsabilité
docs_terms_disclaimers_p = Le service est fourni « en l’état » et « selon disponibilité », sans garantie d’aucune sorte, dans la mesure maximale permise par la loi. Nous ne sommes pas responsables des dommages indirects, accessoires ou consécutifs, ni des contenus publiés par des utilisateurs ou d’autres instances. Rien dans ces Conditions ne limite une responsabilité qui ne peut être limitée par la loi, ni vos droits légaux de consommateur.
docs_terms_changes_h = 10. Modifications
docs_terms_changes_p = Nous pouvons modifier ces Conditions. Si le changement est important, nous vous en informerons dans l’app avant son entrée en vigueur. Continuer à utiliser le service ensuite signifie que vous acceptez les nouvelles Conditions.
docs_terms_contact_h = 11. Contact
docs_terms_contact_lead = Questions sur ces Conditions :
docs_privacy_summary_h = La version courte
docs_privacy_summary_1 = Pas de publicité, pas de scripts de suivi, pas de vente de données.
docs_privacy_summary_2 = Un seul cookie, pour garder votre session ouverte. Les préférences de thème restent dans votre navigateur.
docs_privacy_summary_3 = Les posts publics sont publics et peuvent atteindre les instances tuita fédérées.
docs_privacy_summary_4 = Les messages privés entre comptes de ce serveur sont chiffrés de bout en bout dès que les deux personnes l’activent : ils sont stockés sous forme de texte que le serveur ne peut pas lire, si bien qu’une fuite de la base de données ou un coup d’œil aux données stockées ne révèle rien. Les messages envoyés à des personnes d’autres serveurs, et ceux envoyés avant l’activation du chiffrement, ne sont pas chiffrés de cette manière et les administrateurs peuvent techniquement les lire. Ce serveur fournit aussi l’application qui effectue le chiffrement : un administrateur qui la modifierait pourrait donc capturer ce que vous saisissez.
docs_privacy_summary_5 = Vous pouvez exporter ou supprimer vos données quand vous le voulez.
docs_privacy_controller_h = 1. Qui nous sommes
docs_privacy_controller_p = Cette politique couvre { $name } à l’adresse { $domain }, géré par ses administrateurs d’instance. Ce sont eux qui décident du traitement de vos données.
docs_privacy_collect_h = 2. Ce que nous collectons
docs_privacy_collect_1 = Données de compte : identifiant, nom affiché, mot de passe (stocké uniquement sous forme de hachage argon2id salé) et détails de votre profil : bio, lieu, site web, avatar, bannière, forme et couleur.
docs_privacy_collect_2 = Contenu : posts, images, réponses, j’aime, repartages, signets, abonnements, blocages, masquages et signalements.
docs_privacy_collect_3 = Messages privés : stockés sur ce serveur afin de pouvoir vous être remis, ainsi qu’à l’autre personne. Un message chiffré de bout en bout est stocké sous forme de texte chiffré que nous ne pouvons pas lire ; l’expéditeur, le destinataire et la date restent visibles pour nous. Les autres messages sont stockés en clair. Votre clé de chiffrement n’est stockée ici que sous une forme verrouillée par votre phrase secrète, que nous ne voyons jamais.
docs_privacy_collect_4 = Données de session : pour chaque appareil connecté, l’agent utilisateur du navigateur, l’adresse IP et l’heure de dernière utilisation, afin que vous puissiez consulter et déconnecter vos sessions.
docs_privacy_collect_5 = Données de sécurité éphémères : adresses IP conservées en mémoire quelques minutes pour limiter le débit des requêtes.
docs_privacy_collect_p = Nous ne collectons pas votre e-mail, votre numéro de téléphone, vos contacts ni votre position, sauf si vous les saisissez dans votre profil.
docs_privacy_use_h = 3. Comment nous les utilisons
docs_privacy_use_p = Uniquement pour faire fonctionner le service : afficher votre profil et vos posts, construire les fils et les notifications, remettre les messages, sécuriser les comptes, prévenir les abus et appliquer les Conditions. Le fil « Pour vous » classe les posts selon les j’aime, repartages, réponses et l’ancienneté. Il n’établit pas de profil à votre sujet.
docs_privacy_visibility_h = 4. Ce que les autres peuvent voir
docs_privacy_visibility_p = Votre profil, vos posts, réponses, repartages, j’aime, abonnés et abonnements sont publics pour toute personne connectée à cette instance et pour les instances fédérées. Lorsque quelqu’un partage un lien vers un post ou un profil, des apps comme Discord, Slack ou iMessage peuvent récupérer un aperçu montrant le post public, ses images et les détails publics de votre profil. Les posts derrière un avertissement de contenu ou marqués comme sensibles sont prévisualisés sans leur texte ni leurs images. Les signets, blocages, masquages, signalements et paramètres sont privés. Si vous désactivez « Me montrer dans les suggestions », on peut toujours vous trouver avec votre identifiant exact.
docs_privacy_federation_h = 5. Fédération
docs_privacy_federation_p = Lorsque vous suivez, répondez à, mentionnez, aimez, repartagez ou écrivez à quelqu’un d’une autre instance tuita, nous envoyons à cette instance les données nécessaires : votre profil public, le post ou l’interaction, et le texte du message pour les messages privés. Les messages privés destinés à d’autres instances ne sont pas chiffrés de bout en bout. Les instances de vos abonnés reçoivent vos posts publics et les mises à jour de votre profil. Ces instances sont gérées indépendamment et stockent les données selon leurs propres politiques. Lorsque vous supprimez un post ou votre compte, nous demandons aux instances fédérées de supprimer leurs copies, mais nous ne pouvons pas garantir qu’elles le font.
docs_privacy_cookies_h = 6. Cookies et stockage local
docs_privacy_cookies_p = Nous utilisons un seul cookie de session strictement nécessaire, de type HttpOnly. Vos préférences de thème, de couleur, de taille de texte et d’animations sont stockées dans le stockage local de votre navigateur et ne nous sont jamais envoyées.
docs_privacy_retention_h = 7. Durée de conservation des données
docs_privacy_retention_1 = Le contenu et les données de compte sont conservés jusqu’à ce que vous les supprimiez.
docs_privacy_retention_2 = Les posts supprimés sont immédiatement retirés de la vue publique, et leur texte et leurs images sont effacés.
docs_privacy_retention_3 = Lorsque vous supprimez votre compte, nous supprimons vos posts, j’aime, abonnements, messages et sessions, et anonymisons l’enregistrement du compte.
docs_privacy_retention_4 = Les sessions expirent après 30 jours, ou immédiatement lorsque vous vous déconnectez.
docs_privacy_retention_5 = Les signalements de modération et le journal d’audit de l’administration sont conservés aussi longtemps que nécessaire pour la sécurité du service.
docs_privacy_rights_h = 8. Vos droits
docs_privacy_rights_p = Selon votre lieu de résidence (par exemple au titre du RGPD ou du CCPA), vous pouvez avoir le droit d’accéder à vos données, de les corriger, de les exporter ou de les supprimer, et de vous opposer à leur traitement ou de le limiter. Vous pouvez faire l’essentiel vous-même dans Paramètres → Compte : modifier votre profil, télécharger vos données et supprimer votre compte. Pour tout le reste, contactez :
docs_privacy_rights_end = Vous pouvez aussi déposer une plainte auprès de votre autorité locale de protection des données.
docs_privacy_security_h = 9. Sécurité
docs_privacy_security_p = Les mots de passe sont hachés avec scrypt. Les jetons de session ne sont stockés que sous forme de hachages. Les requêtes entre instances fédérées sont signées avec des clés Ed25519. Aucun système n’est parfaitement sûr, alors utilisez un mot de passe unique.
docs_privacy_children_h = 10. Enfants
docs_privacy_children_p = Le service n’est pas destiné aux enfants en dessous de l’âge minimum indiqué dans les Conditions. Si vous pensez qu’un enfant a un compte, signalez-le.
docs_privacy_changes_h = 11. Modifications
docs_privacy_changes_p = Nous publierons les modifications de cette politique ici et mettrons à jour la date ci-dessus. Les changements importants seront annoncés dans l’app.
docs_privacy_contact_h = 12. Contact
docs_privacy_contact_lead = Questions ou demandes sur la confidentialité :
profile_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
profile_edit = Modifier le profil
profile_more = Plus
profile_copy_link = Copier le lien du profil
profile_mute = Masquer { $handle }
profile_unmute = Ne plus masquer { $handle }
profile_block = Bloquer { $handle }
profile_unblock = Débloquer { $handle }
profile_unblock_button = Débloquer
profile_suspended = Ce compte est suspendu. Seule l’équipe peut le voir.
profile_muted = Masqué
profile_org = Organisation
profile_role_admin = Admin
profile_role_moderator = Modérateur
profile_joined = A rejoint en { $date }
profile_following_label = Abonnements
profile_followers =
    { $count ->
        [one] Abonné
       *[other] Abonnés
    }
profile_counts = Abonnés et abonnements
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Suivi par { $a }
            }
        [2]
            { $others ->
               *[other] Suivi par { $a } et { $b }
            }
       *[other]
            { $others ->
                [one] Suivi par { $a }, { $b } et { $others } autre personne que vous suivez
               *[other] Suivi par { $a }, { $b } et { $others } autres personnes que vous suivez
            }
    }
profile_banner = Bannière du profil
profile_tabs = Sections du profil
profile_tab_posts = Posts
profile_tab_replies = Réponses
profile_tab_media = Médias
profile_tab_likes = J’aime
profile_posts_label = Posts
profile_empty_posts_self_title = Vous n’avez encore rien publié
profile_empty_posts_self_body = Vos posts et repartages apparaîtront ici.
profile_empty_posts_title = { $handle } n’a rien publié
profile_empty_posts_body = Dès que ce sera le cas, ses posts apparaîtront ici.
profile_empty_replies_title = Pas encore de réponses
profile_empty_replies_body = Les réponses apparaîtront ici.
profile_empty_media_title = Pas encore de photos
profile_empty_media_body = Les posts avec images apparaîtront ici.
profile_empty_likes_title = Pas encore de j’aime
profile_empty_likes_body = Les posts aimés apparaîtront ici.
profile_blocked_title = Vous avez bloqué { $handle }
profile_blocked_body = Vous ne verrez pas ses posts, et cette personne ne peut ni vous suivre ni vous écrire.
profile_hidden_title = Vous ne pouvez pas voir ces posts
profile_hidden_body = { $handle } a limité les personnes pouvant voir son profil.
follows_tabs = Listes d’abonnements
follows_tab_followers = Abonnés
follows_tab_following = Abonnements
follows_label_followers = Abonnés
follows_label_following = Abonnements
follows_empty_followers_title = Pas encore d’abonnés
follows_empty_followers_body = Quand des personnes suivront ce compte, elles apparaîtront ici.
follows_empty_following_title = Ne suit personne
follows_empty_following_body = Les comptes suivis apparaîtront ici.
thread_title = Post
thread_doc_title = { $name } : « { $text } »
thread_views =
    { $count ->
        [one] Vue
       *[other] Vues
    }
thread_reposts =
    { $count ->
        [one] Repartage
       *[other] Repartages
    }
thread_quotes =
    { $count ->
        [one] Citation
       *[other] Citations
    }
thread_likes =
    { $count ->
        [one] J’aime
       *[other] J’aime
    }
thread_stats = Statistiques du post
thread_replies = Réponses
thread_hidden = Réponses masquées ({ $count })
thread_hidden_note = L’auteur a masqué ces réponses.
thread_list_likes = Aimé par
thread_list_reposts = Repartagé par
thread_list_quotes = Citations
thread_list_empty = Personne pour l’instant
tag_empty_title = Aucun post avec { $tag }
tag_empty_body = Soyez le premier à l’utiliser.
tag_posts_label = Posts avec { $tag }
bookmarks_title = Signets
bookmarks_label = Posts enregistrés
bookmarks_empty_title = Gardez des posts pour plus tard
bookmarks_empty_body = Touchez l’icône signet d’un post pour le garder ici. Vous seul voyez vos signets.
settings_account = Votre compte
settings_account_desc = Profil et mot de passe
settings_profile = Modifier le profil
settings_profile_desc = Nom, bio, forme de l’avatar et encre
settings_privacy = Confidentialité et sécurité
settings_privacy_desc = Messages, visibilité, mots masqués, comptes bloqués et masqués
settings_notifications = Notifications
settings_notifications_desc = Choisissez les notifications que vous recevez
settings_appearance = Apparence
settings_appearance_desc = Thème, encre, taille du texte, contraste et animations
settings_language = Langue
settings_language_desc = Langue de l’interface
settings_language_note = Enregistrée dans ce navigateur, et sur votre compte si vous êtes connecté.
settings_about = À propos de cette instance
settings_about_desc = Règles, conditions, confidentialité, version et raccourcis clavier
settings_logout = Se déconnecter
settings_account_profile = Profil
settings_account_joined = A rejoint en { $date }
settings_account_edit = Modifier le profil
settings_password_title = Changer le mot de passe
settings_password_current = Mot de passe actuel
settings_password_new = Nouveau mot de passe
settings_password_hint = Au moins 8 caractères. Les autres appareils seront déconnectés.
settings_password_confirm = Confirmez le nouveau mot de passe
settings_password_submit = Mettre à jour le mot de passe
settings_password_mismatch = Les nouveaux mots de passe ne correspondent pas.
settings_password_updated = Mot de passe mis à jour. Les autres sessions ont été déconnectées.
settings_sessions_title = Appareils et sessions
settings_sessions_desc = Déconnectez tout ce que vous ne reconnaissez pas, puis changez votre mot de passe.
settings_device_on = { $browser } sur { $os }
settings_device_unknown = Appareil inconnu
settings_session_current = Cet appareil
settings_session_last_active = Dernière activité : { $time }
settings_session_sign_out = Déconnecter
settings_session_sign_out_device = Déconnecter { $device }
settings_session_signed_out = Session déconnectée
settings_sessions_revoke_all = Déconnecter tous les autres appareils
settings_sessions_revoke_title = Déconnecter les autres appareils ?
settings_sessions_revoke_body = Tous les appareils sauf celui-ci seront déconnectés.
settings_sessions_revoke_confirm = Déconnecter les autres
settings_sessions_revoked =
    { $count ->
        [one] { $count } session déconnectée
       *[other] { $count } sessions déconnectées
    }
settings_profile_preview = Aperçu
settings_profile_preview_alt = Aperçu de l’avatar
settings_profile_uploads = L’envoi d’avatar et de bannière arrivera plus tard. D’ici là, votre avatar et votre bannière générés sont utilisés.
settings_profile_details = Détails
settings_profile_name = Nom affiché
settings_profile_bio = Bio
settings_profile_location = Lieu
settings_profile_website = Site web
settings_profile_shape = Forme de l’avatar
settings_profile_ink = Encre du profil
settings_profile_save = Enregistrer le profil
settings_profile_saved = Profil mis à jour
settings_shape_square = Carré
settings_shape_circle = Cercle
settings_shape_squircle = Carré arrondi
settings_shape_octagon = Octogone
settings_ink_bone = Ivoire
settings_ink_phosphor = Phosphore
settings_ink_amber = Ambre
settings_ink_cyan = Cyan
settings_ink_rose = Rose
settings_dm_title = Messages privés
settings_dm_desc = Qui peut démarrer une conversation avec vous
settings_dm_label = Qui peut vous écrire
settings_dm_everyone = Tout le monde
settings_dm_following = Personnes que je suis
settings_dm_none = Personne
settings_dm_saved = Paramètres des messages enregistrés
settings_discover_title = Visibilité
settings_discoverable = M’afficher dans les suggestions et la recherche
settings_discoverable_hint = On peut toujours vous trouver avec votre identifiant exact
settings_show_client = Afficher l’appareil depuis lequel je publie
settings_show_client_hint = Ajoute l’appareil sous vos nouveaux posts
settings_words_title = Mots masqués
settings_words_desc = Masquez les posts contenant ces mots, expressions ou #hashtags dans vos fils, la recherche et les réponses. Vos propres posts ne sont jamais masqués.
settings_words_label = Mot à masquer
settings_words_placeholder = Ajoutez un mot, une expression ou un #hashtag
settings_words_add = Masquer
settings_words_remove = Ne plus masquer { $word }
settings_words_none = Aucun mot masqué.
settings_words_limit = Vous pouvez masquer jusqu’à { $count } mots.
settings_blocked_title = Comptes bloqués
settings_blocked_desc = Ces personnes ne peuvent ni vous suivre, ni vous écrire, ni voir vos posts ou interagir avec eux. Elles n’en sont pas informées.
settings_blocked_empty = Vous n’avez bloqué personne.
settings_muted_title = Comptes masqués
settings_muted_desc = Vous ne verrez plus ses posts ni ses notifications. Cette personne peut toujours vous suivre et vous écrire.
settings_muted_empty = Vous n’avez masqué personne.
settings_unblock = Débloquer
settings_unmute = Ne plus masquer
settings_unblock_user = Débloquer { $handle }
settings_unmute_user = Ne plus masquer { $handle }
settings_notif_title = Me notifier pour
settings_notif_desc = Les comptes masqués et bloqués ne vous notifient jamais.
settings_notif_like = J’aime
settings_notif_like_hint = Quand quelqu’un aime votre post
settings_notif_repost = Repartages
settings_notif_repost_hint = Quand quelqu’un repartage votre post
settings_notif_reply = Réponses
settings_notif_reply_hint = Quand quelqu’un vous répond
settings_notif_mention = Mentions
settings_notif_mention_hint = Quand quelqu’un vous mentionne
settings_notif_quote = Citations
settings_notif_quote_hint = Quand quelqu’un cite votre post
settings_notif_follow = Nouveaux abonnés
settings_notif_follow_hint = Quand quelqu’un vous suit
settings_desktop_title = Notifications de bureau
settings_desktop_desc = Affichées comme notifications système uniquement lorsque cet onglet est en arrière-plan.
settings_desktop_state = Autorisation : { $state }
settings_desktop_allowed = Autorisée
settings_desktop_blocked = Bloquée
settings_desktop_not_asked = Non demandée
settings_desktop_enable = Activer
settings_desktop_blocked_toast = Les notifications de ce site sont bloquées dans les paramètres de votre navigateur.
settings_desktop_unsupported = Ce navigateur ne prend pas en charge les notifications de bureau.
settings_theme_title = Thème
settings_appearance_note = Enregistré sur cet appareil et appliqué immédiatement.
settings_theme_mode = Thème
settings_theme_system = Système
settings_theme_light = Clair
settings_theme_dark = Sombre
settings_theme_ink = Encre
settings_theme_contrast = Contraste
settings_contrast_normal = Normal
settings_contrast_more = Élevé
settings_reading_title = Texte et animations
settings_theme_text_size = Taille du texte
settings_theme_motion = Animations
settings_motion_system = Système
settings_motion_reduce = Réduites
settings_preview_title = Aperçu
settings_preview_text = Voici à quoi ressembleront les posts avec vos réglages.
settings_about_domain = Domaine
settings_about_version = Version
settings_about_contact = Contact
settings_about_terms = Conditions d’utilisation
settings_about_privacy = Politique de confidentialité
settings_about_source = Code source
settings_about_rules = Règles
explore_search_label = Rechercher des posts, des personnes et des tags
explore_search_placeholder = Rechercher des posts, des personnes, des #tags
explore_clear = Effacer la recherche
explore_tips = Astuces de recherche
explore_tips_title = Opérateurs de recherche
explore_op_from = Posts d’un compte
explore_op_to = Réponses ou mentions adressées à un compte
explore_op_media = Posts avec images (ou has:poll, has:links)
explore_op_since = Publiés à partir d’une date
explore_op_until = Publiés jusqu’à une date
explore_op_phrase = Mots dans cet ordre
explore_op_exclude = Exclure les posts contenant un mot
explore_results = Résultats de recherche
explore_tab_posts = Posts
explore_tab_people = Personnes
explore_tab_media = Médias
explore_people = Personnes
explore_posts = Posts
explore_recent = Recherches récentes
explore_recent_clear = Effacer
explore_trending = Tendances
explore_trend_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
explore_trends_empty = Pas encore de tendances. Utilisez un #hashtag !
explore_suggestions = Suggestions
explore_suggestions_empty = Vous suivez tout le monde. Impressionnant.
explore_no_posts_title = Aucun post pour « { $q } »
explore_no_posts_body = Essayez un autre mot ou hashtag.
explore_no_people_title = Personne pour « { $q } »
explore_no_people_body = Essayez un nom, un identifiant ou @identifiant@instance pour quelqu’un d’une autre instance.
explore_no_media_title = Aucune image pour « { $q } »
explore_no_media_body = Les posts avec images correspondants apparaîtront ici.
notif_filter = Filtre des notifications
notif_tab_all = Toutes
notif_tab_mentions = Mentions
notif_empty_all_title = Pas encore de notifications
notif_empty_all_body = Les j’aime, réponses, repartages et nouveaux abonnés apparaîtront ici.
notif_empty_mentions_title = Pas encore de mentions
notif_empty_mentions_body = Les réponses, mentions et citations de vos posts apparaissent ici.
notif_who_two = { $first } et { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } et { $count } autre
       *[other] { $first }, { $second } et { $count } autres
    }
notif_verb_like =
    { $count ->
        [one] a aimé votre post
       *[other] ont aimé votre post
    }
notif_verb_repost =
    { $count ->
        [one] a repartagé votre post
       *[other] ont repartagé votre post
    }
notif_verb_follow =
    { $count ->
        [one] vous suit désormais
       *[other] vous suivent désormais
    }
notif_verb_follow_request =
    { $count ->
        [one] a demandé à vous suivre
       *[other] ont demandé à vous suivre
    }
notif_verb_reply =
    { $count ->
        [one] vous a répondu
       *[other] vous ont répondu
    }
notif_verb_mention =
    { $count ->
        [one] vous a mentionné
       *[other] vous ont mentionné
    }
notif_verb_quote =
    { $count ->
        [one] a cité votre post
       *[other] ont cité votre post
    }
notif_verb_poll =
    { $count ->
        [one] a terminé un sondage auquel vous avez voté
       *[other] ont terminé un sondage auquel vous avez voté
    }
notif_verb_affiliation =
    { $count ->
        [one] vous a invité à vous affilier à son organisation
       *[other] vous ont invité à vous affilier à leur organisation
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Programmer
composer_post_all = Tout publier
composer_err_type = Ce fichier n’est pas une image ou une vidéo que tuita peut utiliser (PNG, JPEG, GIF, WebP, MP4, WebM ou MOV).
composer_err_image_size = L’image est trop volumineuse (10 Mo max).
composer_err_video_size = Les vidéos peuvent faire jusqu’à { $mb } Mo.
composer_err_max_images = Vous pouvez joindre jusqu’à 4 images.
composer_err_exclusive = Un post peut contenir une vidéo, des images ou un sondage, mais un seul des trois.
composer_extra_placeholder = Ajouter un autre post
composer_extra_label = Post { $n } du fil
composer_extra_remove = Retirer le post { $n } du fil
composer_schedule_chip = Sera publié le { $time }
composer_schedule_clear = Ne pas programmer
composer_add_images = Ajouter des images
composer_add_video = Ajouter une vidéo
composer_add_poll = Ajouter un sondage
composer_add_post = Ajouter un autre post
composer_scheduled = Programmé pour le { $time }
composer_thread_sent = Fil de { $count } posts envoyé
composer_remove_video = Retirer la vidéo
composer_remove_image = Retirer l’image
composer_uploading = Envoi en cours
composer_uploading_video = Envoi de la vidéo… { $percent } %
composer_alt_video = Décrivez cette vidéo
composer_alt_image = Décrivez cette image
composer_alt_label = Description (texte alternatif) de la pièce jointe { $index }
composer_emoji = Ajouter un emoji
poll_length_hours = { $n } h
poll_length_days = { $n } j
poll_choice = Choix { $n }
poll_choice_optional = Choix { $n } (facultatif)
poll_remove_choice = Retirer le choix { $n }
poll_add_choice = Ajouter un choix
poll_length = Durée du sondage
poll_multiple_label = Autoriser plusieurs choix
poll_remove = Retirer le sondage
schedule_title = Programmer le post
schedule_hour = Dans une heure
schedule_evening = Ce soir
schedule_tomorrow = Demain matin
schedule_week = Dans une semaine
schedule_window = Choisissez une heure entre une minute et 90 jours à partir de maintenant.
schedule_custom = Choisir une date et une heure
schedule_set = Valider
schedule_clear = Ne pas programmer
schedule_view = Voir les posts programmés
dropzone_drop = Déposez pour joindre
dropzone_reject = Impossible d’utiliser ce fichier
nav_messages = Messages
nav_lists = Listes
nav_scheduled = Programmés
notify_message = @{ $handle } vous a envoyé un message
shortcut_go_messages = Messages
shortcut_go_lists = Listes
cropper_stage = Zone de recadrage. Faites glisser pour déplacer l’image. Utilisez les flèches pour l’ajuster, et plus et moins pour zoomer.
cropper_hint = Faites glisser pour repositionner. Pincez, faites défiler ou utilisez le curseur pour zoomer.
cropper_zoom = Zoom
cropper_zoom_in = Zoom avant
cropper_zoom_out = Zoom arrière
cropper_rotate = Pivoter
cropper_reset = Réinitialiser
cropper_apply = Appliquer
image_field_avatar = Photo de profil
image_field_banner = Bannière
image_field_edit_avatar = Modifier la photo de profil
image_field_edit_banner = Modifier la bannière
image_field_change = Changer
image_field_remove = Retirer
image_field_hint = PNG, JPEG, GIF ou WebP jusqu’à 10 Mo. Un GIF devient une image fixe.
image_field_not_image = Ce fichier n’est pas une image que tuita peut utiliser (PNG, JPEG, GIF ou WebP).
image_field_too_large = L’image est trop volumineuse (10 Mo max).
image_field_uploading = Envoi en cours
image_field_saved = Image enregistrée.
image_field_removed = Image supprimée.
edit_title = Modifier le post
edit_note = Vous pouvez modifier un post pendant 30 minutes après sa publication. Les gens voient qu’un post a été modifié, ainsi que ses versions précédentes.
edit_save = Enregistrer
edit_saved = Post mis à jour
edit_edited = Modifié
edit_history_title = Historique des modifications
edit_history_current = Version actuelle
edit_history_earlier = Version précédente
edit_history_original = Original
report_post_title = Signaler le post
report_user_title = Signaler { $handle }
report_intro = Quel est le problème ? Les modérateurs de cette instance examineront votre signalement. { $handle } ne verra pas qui l’a signalé.
report_reason = Motif
report_note = Autre chose ? (facultatif)
report_send = Envoyer le signalement
report_sent = Merci. Votre signalement a été envoyé aux modérateurs.
report_duplicate = Vous l’avez déjà signalé. Les modérateurs vont l’examiner.
report_spam = Spam
report_spam_body = Publicités non désirées, arnaques, contenu répétitif ou trompeur
report_harassment = Harcèlement
report_harassment_body = Cibler, harceler ou menacer quelqu’un
report_hate = Haine
report_hate_body = Attaque des personnes pour ce qu’elles sont
report_violence = Violence ou automutilation
report_violence_body = Menaces, apologie de la violence, incitation à l’automutilation
report_misinformation = Désinformation
report_misinformation_body = Fausses affirmations pouvant nuire
report_impersonation = Usurpation d’identité
report_impersonation_body = Se faire passer pour quelqu’un d’autre
report_other = Autre chose
report_other_body = Enfreint les règles d’une autre façon
post_edit = Modifier le post
post_edit_history = Historique des modifications
post_report = Signaler le post
post_hide_reply = Masquer la réponse
post_unhide_reply = Afficher la réponse
post_share_menu = Partager…
post_reply_hidden = Réponse masquée
post_reply_hidden_toast = Réponse masquée
post_reply_unhidden_toast = Réponse de nouveau visible
sensitive_label = Médias sensibles
sensitive_show = Afficher
sensitive_hide = Masquer les médias sensibles
media_label =
    { $count ->
        [one] { $count } pièce jointe
       *[other] { $count } pièces jointes
    }
media_video = Vidéo
media_open = Ouvrir l’image { $index } sur { $count }
lightbox_title = Visionneuse de médias
lightbox_prev = Précédent
lightbox_next = Suivant
poll_vote = Voter
poll_your_vote = Votre vote
poll_votes =
    { $count ->
        [one] { $compact } vote
       *[other] { $compact } votes
    }
poll_final = Résultats finaux
poll_multiple = choix multiple
poll_left_minutes = Encore { $n } min
poll_left_hours = Encore { $n } h
poll_left_days = Encore { $n } j
pull_refresh = Tirez pour actualiser
messages_title = Messages
messages_new = Nouveau message
messages_label = Conversations
messages_empty_title = Démarrer une conversation
messages_empty_body = Les messages que vous envoyez et recevez apparaissent ici.
messages_you_prefix = Vous : { $text }
messages_unread = Non lus : { $count }
messages_new_search = Rechercher des personnes
messages_new_results = Personnes
messages_new_none = Aucun résultat.
chat_load_older = Charger les messages précédents
chat_intro = C’est le début de votre conversation.
chat_label = Messages avec { $name }
chat_delete = Supprimer
chat_seen = Vu
chat_delete_title = Supprimer ce message ?
chat_delete_body = Il disparaît pour vous deux.
chat_doc_title = Discussion avec { $name }
chat_error_title = Impossible d’ouvrir cette conversation
chat_placeholder = Écrire un message
chat_input_label = Message
chat_send = Envoyer
lists_title = Listes
lists_new = Nouvelle liste
lists_edit = Modifier la liste
lists_label = Vos listes
lists_empty_title = Pas encore de listes
lists_empty_body = Regroupez des personnes dans des listes pour lire leurs posts à part.
lists_members =
    { $count ->
        [one] { $compact } membre
       *[other] { $compact } membres
    }
lists_private = Privée
lists_name = Nom
lists_description = Description
lists_private_switch = Liste privée
lists_save = Enregistrer
lists_picker_title = Ajouter { $handle } aux listes
lists_picker_none = Vous n’avez pas encore de listes.
lists_delete = Supprimer la liste
lists_delete_title = Supprimer cette liste ?
lists_delete_body = « { $name } » sera supprimée. Les personnes qu’elle contient ne sont pas touchées.
lists_by = Liste de { $handle }
lists_tab_posts = Posts
lists_tab_members = Membres
lists_tabs = Sections de la liste
lists_error_title = Impossible d’ouvrir cette liste
lists_posts_label = Posts de la liste
lists_posts_empty_title = Rien pour l’instant
lists_posts_empty_body = Les posts des membres de cette liste apparaissent ici.
lists_members_label = Membres de la liste
lists_members_empty_title = Aucun membre
lists_members_empty_body = Ajoutez des personnes depuis leurs profils.
lists_remove = Retirer
scheduled_title = Posts programmés
scheduled_label = Posts programmés
scheduled_status_pending = Programmé
scheduled_status_published = Publié
scheduled_status_failed = Échec
scheduled_cw = Avertissement de contenu : { $text }
scheduled_poll = Sondage à { $count } choix, durée : { $hours } h
scheduled_change_time = Changer l’heure
scheduled_post_now = Publier maintenant
scheduled_remove = Retirer
scheduled_window_error = Choisissez une heure entre une minute et 90 jours à partir de maintenant.
scheduled_when = Date de publication
scheduled_zone = Fuseau horaire : { $zone }
scheduled_save = Enregistrer
scheduled_posted = Publié.
scheduled_cancel_title = Annuler ce post programmé ?
scheduled_cancel_body = Il ne sera pas publié.
scheduled_cancel_confirm = Abandonner le post
scheduled_empty_title = Rien de programmé
scheduled_empty_body = Les posts que vous programmez attendent ici leur heure.
profile_message = Message
profile_add_to_lists = Ajouter aux listes
media_alt = ALT
profile_report = Signaler { $handle }
chart_show_table = Afficher { $title } sous forme de tableau
chart_col_day = Jour
chart_col_hour = Heure (UTC)
chart_empty = { $title } : rien à afficher pour l’instant.
chart_bar_summary = { $title } : { $total } au total, le plus le { $label } ({ $peak }).
chart_spark_summary = { $title } : { $total } au total, { $last } le { $label }.
chart_heat_summary = { $title } : { $total } au total, pic d’activité le { $label } à { $hour } ({ $peak }).
chart_heat_less = Moins
chart_heat_more = Plus
announce_region = Annonces
announce_dismiss = Fermer l’annonce
announce_more = En savoir plus
announce_tone_info = Information
announce_tone_success = Bonne nouvelle
announce_tone_warning = Attention
admin_stat_people = Personnes
admin_stat_people_sub = { $count } actifs cette semaine
admin_stat_posts_today = Posts, 24 h
admin_stat_posts_total = { $count } au total
admin_stat_reports = Signalements ouverts
admin_stat_reports_open = À examiner
admin_stat_reports_clear = Rien à signaler
admin_stat_signups_today = Inscriptions aujourd’hui
admin_stat_signups_sub = { $count } en 14 jours
admin_stat_suspended = Suspendus
admin_stat_suspended_sub = comptes
admin_chart_posts = Posts par jour
admin_chart_signups = Inscriptions par jour
admin_chart_reports = Signalements par jour
admin_chart_heat = Activité par jour de la semaine et heure
nav_admin = Administration
admin_title = Administration
admin_sections = Sections d’administration
admin_tab_overview = Aperçu
admin_tab_reports = Signalements
admin_tab_users = Personnes
admin_tab_instance = Instance
admin_tab_federation = Fédération
admin_tab_announcements = Annonces
admin_tab_invites = Invitations
admin_tab_filter = Filtre de contenu
admin_tab_audit = Journal d’audit
admin_denied_title = Réservé à l’équipe
admin_denied_body = Cet espace est réservé aux administrateurs et modérateurs de cette instance.
admin_denied_home = Retour à l’accueil
admin_unknown = inconnu
admin_tag_active = Actif
admin_tag_suspended = Suspendu
admin_tag_deleted = Supprimé
admin_tag_org = Organisation
admin_audit_system = système
admin_audit_empty = Rien d’enregistré pour l’instant
admin_report_status = Statut du signalement
admin_report_open = Ouverts
admin_report_resolved = Traités
admin_reports_empty_open = Aucun signalement ouvert
admin_reports_empty_open_body = Quand des personnes signalent des posts ou des comptes, ils apparaissent ici.
admin_reports_empty_resolved = Rien de traité pour l’instant
admin_report_by = signalé par { $handle }
admin_report_same_target = { $count } signalements ouverts
admin_report_media =
    { $count ->
        [one] { $count } pièce jointe
       *[other] { $count } pièces jointes
    }
admin_report_poll = Contient un sondage
admin_report_dismiss = Ignorer
admin_report_delete_post = Supprimer le post
admin_report_suspend = Suspendre le compte
admin_report_delete_suspend = Supprimer et suspendre
admin_report_confirm_title = Confirmer l’action ?
admin_report_confirm_body = { $action }. Cette action est consignée dans le journal d’audit.
admin_report_confirm = Confirmer
admin_report_resolution = { $action } par { $handle } · { $time }
admin_res_dismiss = Ignoré
admin_res_delete_post = Post supprimé
admin_res_suspend_user = Compte suspendu
admin_res_delete_and_suspend = Post supprimé et compte suspendu
admin_users_search = Rechercher des personnes
admin_users_search_placeholder = Nom ou identifiant
admin_users_show = Afficher
admin_users_empty = Aucune personne trouvée
admin_filter_local = Locaux
admin_filter_remote = Distants
admin_filter_staff = Équipe
admin_filter_verified = Vérifiés
admin_filter_suspended = Suspendus
admin_filter_all = Tout le monde
admin_user_details = Détails de { $handle }
admin_user_menu = Gérer { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } post
       *[other] { $compact } posts
    }
admin_user_followers =
    { $count ->
        [one] { $compact } abonné
       *[other] { $compact } abonnés
    }
admin_user_joined = inscription : { $time }
admin_user_active = activité : { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } signalement ouvert
       *[other] { $compact } signalements ouverts
    }
admin_act_verify = Vérifier
admin_act_unverify = Retirer la vérification
admin_act_org = Définir comme organisation
admin_act_unorg = Retirer le statut d’organisation
admin_act_role_user = Retirer le rôle d’équipe
admin_act_role_moderator = Nommer modérateur
admin_act_role_admin = Nommer administrateur
admin_act_suspend = Suspendre
admin_act_unsuspend = Lever la suspension
admin_act_purge = Supprimer tous les posts
admin_act_rename = Changer l’identifiant
admin_act_reset_name = Réinitialiser le nom affiché
admin_act_clear_bio = Effacer la bio
admin_act_delete = Supprimer le compte
admin_done_verified = { $handle } est vérifié
admin_done_unverified = Vérification retirée
admin_done_org = { $handle } est désormais une organisation
admin_done_unorg = Statut d’organisation retiré
admin_done_role = Rôle mis à jour
admin_done_suspended = { $handle } suspendu
admin_done_unsuspended = Suspension de { $handle } levée
admin_done_purged =
    { $count ->
        [one] { $count } post supprimé
       *[other] { $count } posts supprimés
    }
admin_done_name_reset = Nom affiché réinitialisé
admin_done_bio_cleared = Bio effacée
admin_done_deleted = { $handle } supprimé
admin_done_renamed = Désormais { $handle }
admin_role_phrase_user = un utilisateur ordinaire
admin_role_phrase_moderator = un modérateur
admin_role_phrase_admin = un administrateur
admin_role_title = Changer le rôle de { $handle } ?
admin_role_body = Cette personne deviendra { $role }.
admin_role_confirm = Changer le rôle
admin_unorg_title = Retirer le statut d’organisation de { $handle } ?
admin_unorg_body = Toutes ses affiliations prennent fin.
admin_unorg_confirm = Retirer
admin_suspend_title = Suspendre { $handle } ?
admin_suspend_body = Cette personne est déconnectée et son contenu reste masqué jusqu’à la levée de la suspension.
admin_suspend_confirm = Suspendre
admin_purge_title = Supprimer tous les posts de { $handle } ?
admin_purge_body = Cette action est irréversible.
admin_purge_confirm = Tout supprimer
admin_reset_name_title = Réinitialiser le nom affiché de { $handle } ?
admin_reset_name_body = Il sera remplacé par son identifiant.
admin_reset_name_confirm = Réinitialiser
admin_delete_title = Supprimer { $handle } ?
admin_delete_body = Ses posts, abonnements et messages sont supprimés et le compte est anonymisé. Cette action est irréversible.
admin_delete_confirm = Supprimer le compte
admin_rename_title = Changer l’identifiant de { $handle }
admin_rename_note = Les liens vers l’ancien identifiant cessent de fonctionner.
admin_rename_handle = Nouvel identifiant
admin_rename_reset_name = Réinitialiser aussi le nom affiché
admin_rename_save = Changer l’identifiant
admin_detail_title = Détails du compte
admin_detail_invite = Invitation { $code }
admin_detail_posts = Posts
admin_detail_followers = Abonnés
admin_detail_following = Abonnements
admin_detail_open_reports = Signalements ouverts
admin_detail_sessions = Sessions
admin_detail_passkeys = Clés d’accès
admin_detail_tokens = Jetons d’API
admin_detail_joined = Inscription : { $time }
admin_detail_recent_posts = Posts récents
admin_detail_no_posts = Aucun post.
admin_detail_reports = Signalements concernant ce compte
admin_detail_no_reports = Aucun signalement.
admin_detail_history = Historique de modération
admin_detail_nothing = Rien pour l’instant.
admin_instance_title = Instance
admin_instance_name = Nom de l’instance
admin_instance_description = Description
admin_instance_contact = E-mail de contact
admin_instance_contact_hint = Affiché dans les pages Conditions et Confidentialité
admin_instance_min_age = Âge minimum
admin_instance_rules = Règles
admin_instance_rules_hint = Une règle par ligne
admin_instance_preview = Aperçu des conditions
admin_instance_save = Enregistrer
admin_instance_saved = Paramètres de l’instance enregistrés
admin_instance_template_note = Les conditions d’utilisation et la politique de confidentialité sont des modèles qui utilisent ces valeurs. Faites-les relire pour votre juridiction avant d’ouvrir l’instance au public.
admin_signups_title = Inscriptions
admin_signups_desc = Qui peut créer un compte sur cette instance
admin_signups_open = Ouvertes
admin_signups_open_hint = Tout le monde peut créer un compte.
admin_signups_invite = Sur invitation
admin_signups_invite_hint = Il faut un code d’invitation, créé dans l’onglet Invitations.
admin_signups_closed = Fermées
admin_signups_closed_hint = Personne ne peut s’inscrire.
admin_filter_title = Filtre de contenu
admin_filter_desc = Rien n’est bloqué tant que vous n’ajoutez pas de mots ci-dessous. Les mots bloqués s’appliquent aux identifiants, noms affichés, lieux, bios, posts et noms de listes, y compris les orthographes déguisées. Les identifiants réservés comme « admin » ne peuvent pas être enregistrés.
admin_filter_blocked = Mots bloqués
admin_filter_blocked_hint = Un par ligne, détectés à l’intérieur des mots
admin_filter_allowed = Mots autorisés
admin_filter_allowed_hint = Exceptions aux mots bloqués, un par ligne
admin_filter_reserved = Identifiants réservés supplémentaires
admin_filter_reserved_hint = Un par ligne
admin_filter_save = Enregistrer le filtre
admin_filter_saved = Filtre enregistré
admin_filter_field_handle = identifiant
admin_filter_field_name = nom
admin_filter_field_location = lieu
admin_filter_field_bio = bio
admin_scan_title = Analyser les comptes
admin_scan_desc = Trouvez les comptes dont l’identifiant, le nom ou la bio enfreignent le filtre.
admin_scan_run = Analyser maintenant
admin_scan_clean = Aucun compte n’enfreint le filtre.
admin_scan_found =
    { $count ->
        [one] { $count } compte demande votre attention
       *[other] { $count } comptes demandent votre attention
    }
admin_scan_reason = Le champ { $field } contient { $reason }
admin_ann_title = Nouvelle annonce
admin_ann_desc = Une bannière en haut de l’écran de tout le monde jusqu’à son expiration ou jusqu’à ce que vous y mettiez fin. Les gens peuvent la fermer.
admin_ann_text = Message
admin_ann_link = Lien
admin_ann_link_hint = Facultatif : https://… ou /chemin
admin_ann_tone = Style
admin_ann_tone_info = Info
admin_ann_tone_success = Bonne nouvelle
admin_ann_tone_warning = Attention
admin_ann_duration = Durée
admin_ann_days =
    { $count ->
        [one] { $count } jour
       *[other] { $count } jours
    }
admin_ann_week = 1 semaine
admin_ann_until_ended = Jusqu’à arrêt manuel
admin_ann_publish = Publier
admin_ann_published = Annonce publiée
admin_ann_list = Annonces publiées
admin_ann_none = Pas encore d’annonces.
admin_ann_end = Terminer
admin_ann_ended = terminée
admin_ann_expired = expirée
admin_ann_inactive = Non affichée
admin_ann_until = jusqu’au { $date }
admin_inv_title = Codes d’invitation
admin_inv_desc = Permettez à certaines personnes de s’inscrire tant que les inscriptions sont sur invitation.
admin_inv_note = Note (pour qui)
admin_inv_uses = Utilisations
admin_inv_use_count =
    { $count ->
        [one] { $count } utilisation
       *[other] { $count } utilisations
    }
admin_inv_expiry = Expire
admin_inv_week = Dans 1 semaine
admin_inv_days =
    { $count ->
        [one] Dans { $count } jour
       *[other] Dans { $count } jours
    }
admin_inv_never = Jamais
admin_inv_create = Créer le code
admin_inv_created = Code { $code } créé
admin_inv_created_copied = Code { $code } créé et copié
admin_inv_list = Codes
admin_inv_none = Pas encore de codes d’invitation.
admin_inv_state_active = Actif
admin_inv_state_revoked = Révoqué
admin_inv_state_used = Épuisé
admin_inv_state_expired = Expiré
admin_inv_used = { $uses } utilisé(s) sur { $max }
admin_inv_expires = expire le { $date }
admin_inv_copy = Copier le code { $code }
admin_inv_copied = Code copié
admin_inv_copy_failed = Copie impossible. Le code est { $code }.
admin_inv_revoke = Révoquer
admin_inv_revoke_title = Révoquer cette invitation ?
admin_inv_revoke_body = Les personnes qui se sont déjà inscrites avec ce code conservent leur compte.
settings_devices = Appareils et sessions
settings_devices_desc = Voyez où vous êtes connecté et déconnectez-vous à distance
settings_passkeys = Clés d’accès
settings_passkeys_desc = Connectez-vous avec votre empreinte digitale, votre visage, le verrouillage d’écran ou une clé de sécurité
settings_affiliation = Affiliation
settings_affiliation_desc = Les organisations dont vous faites partie, ou les comptes de votre organisation
settings_bots = Bots et API
settings_bots_desc = Jetons d’API et libellé « Automatisé »
settings_saved = Enregistré
passkey_title = Vos clés d’accès
passkey_desc = Connectez-vous avec votre empreinte digitale, votre visage, le verrouillage d’écran ou une clé de sécurité au lieu d’un mot de passe. Les clés d’accès ne peuvent pas être hameçonnées, et aucun secret n’est stocké sur notre serveur.
passkey_none = Pas encore de clé d’accès.
passkey_add = Ajouter une clé d’accès
passkey_added = Clé d’accès « { $name } » ajoutée
passkey_default_name = Clé d’accès
passkey_added_ago = ajout : { $time }
passkey_used_ago = dernière utilisation : { $time }
passkey_never_used = jamais utilisée
passkey_rename = Renommer
passkey_rename_label = Renommer la clé d’accès { $name }
passkey_rename_title = Renommer la clé d’accès
passkey_name = Nom
passkey_rename_save = Enregistrer
passkey_remove = Retirer
passkey_remove_label = Supprimer la clé d’accès { $name }
passkey_remove_title = Supprimer « { $name } » ?
passkey_remove_body = Vous ne pourrez plus vous connecter avec cette clé d’accès. Pensez aussi à la supprimer de votre appareil ou de votre gestionnaire de mots de passe.
passkey_removed = Clé d’accès supprimée
passkey_unsupported = Ce navigateur ne peut pas utiliser de clés d’accès sur cette adresse.
passkey_needs_domain = Les clés d’accès nécessitent que ce site ait un nom de domaine, et non une adresse IP.
passkey_already_here = Cet appareil a déjà une clé d’accès pour ce compte.
token_scope_read = Lecture
token_scope_write = Publier et interagir
token_scope_messages = Messages privés
token_scope_read_hint = Fils, notifications, profils, recherche et flux en temps réel
token_scope_write_hint = Publication, suppression, j’aime, repartages, signets, abonnements et envois de fichiers
token_scope_messages_hint = Lecture et envoi de messages privés
token_title = Jetons d’API
token_desc = Les jetons permettent aux scripts et bots d’utiliser votre compte via l’API. Traitez-les comme des mots de passe : quiconque en possède un peut agir en votre nom selon ses autorisations.
token_none = Pas encore de jeton.
token_name = Nom du jeton
token_name_hint = Par exemple : bot météo
token_scopes = Autorisations
token_create = Créer un jeton
token_used_ago = dernière utilisation : { $time }
token_never_used = jamais utilisé
token_revoke = Révoquer
token_revoke_label = Révoquer le jeton { $name }
token_revoke_title = Révoquer « { $name } » ?
token_revoke_body = Tout ce qui l’utilise cessera de fonctionner.
token_revoked = Jeton révoqué
token_secret_title = Votre nouveau jeton
token_secret_warning = Copiez-le maintenant. Pour votre sécurité, il ne sera plus affiché.
token_secret_label = Nouveau jeton d’API
token_copy = Copier
token_copied = Jeton copié
token_copy_failed = Copie impossible. Sélectionnez le jeton et copiez-le à la main.
token_done = Terminé
bot_title = Compte automatisé
bot_desc = Affiche un libellé « Automatisé » à côté du nom, pour que l’on sache qu’un bot publie.
bot_switch = Ce compte est automatisé
bot_owner = Géré par
bot_owner_hint = L’identifiant de la personne ou de l’organisation derrière ce compte. Facultatif.
bot_owner_save = Enregistrer
bot_docs = Documentation développeurs
bot_docs_desc = Points d’accès, événements en temps réel et bots d’exemple
aff_yours_title = Votre affiliation
aff_yours_desc = Une organisation peut vous inviter à afficher son badge à côté de votre nom. Rien ne change tant que vous n’acceptez pas, et vous pouvez partir à tout moment.
aff_none = Vous n’êtes affilié à aucune organisation.
aff_invites_title = Invitations
aff_accept = Accepter
aff_decline = Refuser
aff_leave = Quitter
aff_leave_title = Quitter cette organisation ?
aff_leave_body = Son badge disparaît de votre profil.
aff_remove_title = Retirer { $handle } ?
aff_remove_body = Votre badge disparaît de son profil.
aff_remove = Retirer
aff_remove_label = Retirer { $handle }
aff_ended = Affiliation terminée
aff_declined = Invitation refusée
aff_accepted = Affiliation ajoutée
aff_invited = Invitation envoyée
aff_org_title = Vos affiliés
aff_org_desc = Invitez des personnes à afficher votre badge à côté de leur nom. Elles doivent accepter.
aff_handle = Identifiant
aff_label = Libellé
aff_label_hint = Facultatif, par exemple « Ingénieur »
aff_invite = Envoyer l’invitation
aff_members_none = Pas encore d’affiliés.
aff_pending = Invité
aff_not_org = Vous êtes une organisation ? Demandez à un administrateur de cette instance de marquer votre compte.
auth_passkey = Se connecter avec une clé d’accès
auth_or = ou
auth_invite = Code d’invitation
auth_invite_hint = Cette instance n’accepte de nouvelles personnes que sur invitation.
auth_invite_required = Saisissez le code d’invitation qui vous a été donné.
landing_developers = Développeurs
landing_invite = Les inscriptions se font uniquement sur invitation. Vous avez un code ? Créez votre compte.
dev_title = Développeurs
dev_hero = Bots et API
dev_hero_sub = Créez des bots, des scripts et des intégrations pour { $name }
dev_intro_h = Développer pour { $name }
dev_intro_p = Tout ce que font les apps tuita passe par une petite API JSON, que les bots et les scripts peuvent aussi utiliser. Il vous faut un compte sur { $name } et un jeton d’API personnel. Pas d’enregistrement d’app ni de parcours OAuth compliqué.
dev_intro_1 = Créez un compte pour votre bot, ou utilisez le vôtre.
dev_intro_2 = Dans Paramètres, Bots et API, activez « Ceci est un compte automatisé » pour que les gens voient un libellé « Automatisé », et indiquez qui le gère.
dev_intro_3 = Créez un jeton avec les autorisations nécessaires et gardez-le secret.
dev_auth_h = Authentification
dev_auth_p = Envoyez le jeton dans l’en-tête Authorization. Chaque requête et chaque réponse est en JSON (UTF-8), sauf les envois de fichiers, qui sont en multipart.
dev_auth_prefix = Un jeton commence par tuita_ suivi de 43 lettres, chiffres, tirets et tirets bas. Seul un hachage est stocké : un jeton perdu ne peut pas être réaffiché, il faut en créer un nouveau. Un jeton possède une ou plusieurs des autorisations suivantes :
dev_auth_refused = Un jeton ne peut pas modifier les mots de passe, sessions, jetons ou clés d’accès, ne peut pas exporter ni supprimer le compte, et ne peut pas utiliser l’API d’administration. Cela nécessite une vraie connexion sur le site web et renvoie 403.
dev_auth_errors = Les erreurs sont renvoyées en JSON avec un message error et un statut 4xx. 401 signifie que le jeton est incorrect ou révoqué, 403 une autorisation manquante ou une route refusée, et 429 s’accompagne d’un en-tête Retry-After.
dev_post_h = Votre premier post
dev_post_p = Envoyez d’abord les images et vidéos avec POST /api/v1/media (multipart), puis joignez-les par identifiant dans mediaIds. Une réponse définit replyTo, une citation définit quoteOf.
dev_endpoints_h = Points d’accès
dev_endpoints_intro = Les chemins entre accolades sont des paramètres. Les listes se paginent avec ?before= réglé sur la valeur suivante de la page précédente ; limit va de 1 à 40. La colonne Autorisation indique l’autorisation requise pour le jeton.
dev_col_method = Méthode
dev_col_path = Chemin
dev_col_scope = Autorisation
dev_col_what = Fonction
dev_ep_me = Le compte auquel appartient le jeton
dev_ep_patch_me = Modifier le profil et les paramètres de ce compte
dev_ep_bot = Activer ou désactiver le libellé « Automatisé » et indiquer la personne qui gère le bot
dev_ep_following = Le fil d’accueil des personnes que vous suivez
dev_ep_foryou = Le fil « Pour vous » ; pagination avec ?cursor=
dev_ep_notifications = Notifications (?filter=mentions pour les réponses, mentions et citations)
dev_ep_post = Un post avec son fil et ses réponses
dev_ep_user = Un profil (identifiant@domaine pour les comptes d’autres serveurs)
dev_ep_user_posts = Les posts de quelqu’un (?tab=replies, media ou likes ; avec media aussi ?kind=photo ou video)
dev_ep_search = Rechercher des posts, des personnes ou des médias, avec des opérateurs comme from: et has:media
dev_ep_trends = Les tags les plus utilisés la semaine dernière
dev_ep_stream = Événements en temps réel (Server-Sent Events)
dev_ep_create = Écrire un post
dev_ep_thread = Écrire de 2 à 25 posts sous forme de fil
dev_ep_delete = Supprimer un de vos posts
dev_ep_like = Aimer un post (DELETE pour annuler). Aussi /repost et /bookmark
dev_ep_follow = Suivre quelqu’un (DELETE pour ne plus suivre)
dev_ep_media = Envoyer une image ou une vidéo en multipart ; la réponse contient l’identifiant à joindre
dev_ep_schedule = Programmer un post pour plus tard
dev_ep_report = Signaler un post ou un compte aux modérateurs
dev_ep_conversations = Vos conversations de messages privés
dev_ep_message = Envoyer un message privé ; le corps est du JSON avec un champ text
dev_realtime_h = Événements en temps réel
dev_realtime_p = GET /api/v1/stream est un flux Server-Sent Events. Il envoie un événement notification pour chaque j’aime, repartage, abonnement, réponse, mention ou citation, un événement message pour les messages privés et des badges avec les compteurs de non lus. Il envoie une ligne de commentaire toutes les 25 secondes et demande aux clients de réessayer après 5 secondes ; reconnectez-vous si la connexion est interrompue.
dev_example_h = Exemple : un bot qui répond aux mentions
dev_example_node = Node 22 ou plus récent, sans paquets :
dev_example_python = La même chose en Python, avec requests uniquement :
dev_limits_h = Limites
dev_limit_rate = Chaque jeton peut faire 600 requêtes par 10 minutes. Au-delà, le serveur répond 429 avec un en-tête Retry-After.
dev_limit_tokens = Un compte peut avoir jusqu’à 20 jetons et en créer jusqu’à 20 par heure.
dev_limit_posting = La publication, les abonnements, les signalements et les autres actions ont chacun leurs propres limites par compte.
dev_rules_h = Règles pour les bots
dev_rule_1 = Activez le libellé Automatisé et indiquez qui gère le bot.
dev_rule_2 = Ne répondez ou n’écrivez qu’aux personnes qui l’ont demandé : elles ont mentionné le bot, l’ont suivi ou lui ont écrit en premier. N’envoyez ni mentions ni messages privés non sollicités.
dev_rule_3 = Restez raisonnable : tenez-vous bien en dessous des limites ci-dessus.
dev_rule_4 = Respectez les conditions et règles de l’instance. Les modérateurs peuvent suspendre les bots comme n’importe quel compte.
admin_act_log_admin_bootstrap = Premier administrateur configuré
admin_act_log_announcement_create = Annonce publiée
admin_act_log_announcement_end = Annonce terminée
admin_act_log_instance_settings = Paramètres de l’instance modifiés
admin_act_log_invite_create = Invitation créée
admin_act_log_invite_revoke = Invitation révoquée
admin_act_log_post_delete = Post supprimé
admin_act_log_report_dismiss = Signalement ignoré
admin_act_log_user_delete = Compte supprimé
admin_act_log_user_org = Défini comme organisation
admin_act_log_user_unorg = Statut d’organisation retiré
admin_act_log_user_purge_posts = Tous les posts supprimés
admin_act_log_user_rename = Compte renommé
admin_act_log_user_role = Rôle modifié
admin_act_log_user_suspend = Compte suspendu
admin_act_log_user_unsuspend = Suspension du compte levée
admin_act_log_user_unverify = Vérification retirée
admin_act_log_user_verify = Compte vérifié
admin_act_log_domain_block = Instance bloquée
admin_act_log_domain_unblock = Instance débloquée
admin_act_log_federation_retry = Nouvel essai de livraison demandé
admin_fed_stat_domains = Instances connues
admin_fed_stat_domains_sub = vues par cette instance
admin_fed_stat_accounts = Comptes distants
admin_fed_stat_accounts_sub = venus d’autres instances
admin_fed_stat_posts = Posts distants
admin_fed_stat_posts_sub = stockés localement
admin_fed_stat_queue = En attente
admin_fed_stat_queue_sub = livraisons en attente
admin_fed_queue_title = File de livraison
admin_fed_queue_desc = Activités signées en attente d’être transmises à d’autres instances. Celles qui échouent sont retentées avec des délais croissants et abandonnées après 12 essais.
admin_fed_queue_empty = Rien n’est en attente. Toutes les livraisons ont été envoyées.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } livraison en attente
       *[other] { $compact } livraisons en attente
    }
admin_fed_queue_due = { $count } à traiter maintenant
admin_fed_queue_oldest = Plus ancienne en file : { $date }
admin_fed_retry = Réessayer maintenant
admin_fed_retry_done =
    { $count ->
        [one] { $compact } livraison va être retentée maintenant.
       *[other] { $compact } livraisons vont être retentées maintenant.
    }
admin_fed_failing = Instances en échec
admin_fed_failing_none = Aucune instance en échec.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } livraison
       *[other] { $compact } livraisons
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } essai
       *[other] { $compact } essais
    }
admin_fed_failing_next = prochain essai le { $date }
admin_fed_failing_error = Dernière erreur
admin_fed_failing_no_error = Aucune erreur enregistrée
admin_fed_actor_title = Acteur de l’instance
admin_fed_actor_desc = Les autres instances vérifient avec cette clé les requêtes que cette instance signe.
admin_fed_actor_id = Acteur
admin_fed_actor_key = ID de la clé
admin_fed_actor_fingerprint = Empreinte SHA-256
admin_fed_actor_copy = Copier l’empreinte
admin_fed_actor_copied = Empreinte copiée
admin_fed_actor_copy_failed = Copie impossible. L’empreinte est { $fingerprint }
admin_fed_blocks_title = Instances bloquées
admin_fed_blocks_desc = Limiter masque les comptes d’une instance dans les fils publics et la recherche. Suspendre les masque partout, ignore ce qu’elle envoie et arrête les livraisons vers elle.
admin_fed_domain = Domaine
admin_fed_domain_hint = example.social, ou un lien vers n’importe quelle page de ce domaine
admin_fed_severity = Niveau
admin_fed_silence = Limiter
admin_fed_suspend = Suspendre
admin_fed_reason = Motif
admin_fed_reason_hint = Facultatif, visible uniquement de l’équipe
admin_fed_block = Bloquer l’instance
admin_fed_blocked = { $domain } est bloquée
admin_fed_none = Aucune instance n’est bloquée.
admin_fed_list = Blocages
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } compte connu
       *[other] { $compact } comptes connus
    }
admin_fed_row_no_reason = Aucun motif indiqué
admin_fed_row_changed = { $domain } est maintenant réglée sur { $severity }
admin_fed_remove = Retirer
admin_fed_remove_label = Lever le blocage de { $domain }
admin_fed_remove_title = Lever le blocage de { $domain } ?
admin_fed_remove_body = Ses comptes et ses posts reviennent et les livraisons vers elle reprennent.
admin_fed_removed = { $domain } n’est plus bloquée
remote_domain_title = Compte sur { $domain }
remote_view_original = Voir sur { $domain }
visibility_unlisted = Non listé
visibility_unlisted_hint = Non affiché dans les fils publics
visibility_followers = Abonnés uniquement
visibility_followers_hint = Seuls les abonnés peuvent le voir
visibility_direct = Personnes mentionnées
visibility_direct_hint = Seules les personnes mentionnées peuvent le voir
follow_requested = En attente
follow_cancel_request = Annuler la demande d’abonnement
follow_requested_toast = Demande d’abonnement envoyée à { $handle }
follow_cancelled_toast = Demande d’abonnement annulée
profile_locked = Approuve les abonnés
profile_locked_hint = Les demandes d’abonnement doivent être approuvées par ce compte.
profile_original = Voir le profil sur { $domain }
profile_remote_note = Une copie de { $domain }. Les compteurs et les posts sont ceux que cette instance a vus.
remote_lookup_loading = Recherche de { $handle } sur { $domain }…
remote_lookup_notfound_title = Compte introuvable
remote_lookup_notfound_body = { $handle } n’existe pas, ou { $domain } ne le partage pas.
remote_lookup_failed_title = Impossible de charger { $handle }
explore_lookup = Recherche de { $target }…
explore_lookup_empty_body = Ce compte ou ce post est introuvable. L’autre instance est peut-être hors ligne, ou ne le partage pas.
language_label = Langue
oauth_title = Autoriser une app
oauth_loading = Vérification de la demande…
oauth_wants = Cette app souhaite utiliser votre compte { $handle }.
oauth_name_warning = N’importe qui peut choisir n’importe quel nom d’app. N’approuvez que les apps de confiance.
oauth_website = Site web
oauth_allow = Elle pourra
oauth_scope_read = Lire votre profil, vos posts, abonnements, listes, fils et notifications, ainsi que les messages privés qui ne sont pas chiffrés de bout en bout.
oauth_scope_write = Publier, modifier et supprimer à votre place, aimer, suivre, bloquer, masquer et modifier votre profil et vos paramètres.
oauth_scope_messages = Lire et envoyer vos messages privés.
oauth_push_note = Les notifications push ne sont pas prises en charge. Une app qui en demande n’en obtient pas.
oauth_redirect = Après votre réponse, vous serez redirigé vers
oauth_redirect_oob = L’app ne reçoit aucune redirection. Un code s’affiche ici pour que vous le colliez dans l’app.
oauth_approve = Autoriser
oauth_deny = Refuser
oauth_switch = Utiliser un autre compte
oauth_sent_approved = Autorisé. Retour vers { $host } en cours. Si rien ne se passe, fermez cet onglet.
oauth_sent_denied = Refusé. Retour vers { $host } en cours. Si rien ne se passe, fermez cet onglet.
oauth_denied = Refusé. Rien n’a été partagé avec l’app.
oauth_code_title = Code d’autorisation
oauth_code_body = Collez ce code dans l’app. Il ne fonctionne qu’une fois et expire dans 10 minutes.
oauth_code_copy = Copier le code
oauth_code_copied = Code copié
oauth_code_copy_failed = Copie impossible. Sélectionnez le code et copiez-le à la main.
oauth_error_title = Cette demande ne peut pas être autorisée
oauth_error_note = Rien n’a été envoyé à l’app.
oauth_error_invalid_request = La demande est mal formée, ou elle demande un retour vers une adresse que l’app n’a pas enregistrée. Retournez dans l’app et réessayez.
oauth_error_invalid_client = Cette app n’est pas enregistrée ici.
oauth_error_invalid_scope = L’app demande un accès que tuita ne propose pas, ou plus que ce pour quoi elle est enregistrée.
oauth_error_unsupported_response_type = L’app demande un type de connexion que tuita ne prend pas en charge.
oauth_error_rate = Vous avez répondu à trop de demandes. Attendez quelques minutes et réessayez.
oauth_error_unreachable = Impossible de joindre tuita. Vérifiez votre connexion et réessayez.
oauth_home = Aller sur tuita
share_send_chat = Envoyer par message
share_download = Télécharger le média
share_download_n = Télécharger le média { $number }
share_chat_title = Envoyer le post par message
share_chat_sent = Envoyé à { $handle }
hover_card_label = Carte de profil de { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Type de média
profile_media_all = Tous les médias
profile_media_photos = Photos
profile_media_videos = Vidéos
profile_empty_videos_title = Aucune vidéo pour l’instant
profile_empty_videos_body = Les posts avec des vidéos apparaîtront ici.
account_add_existing = Ajouter un compte existant
rail_label = Recherche et suggestions
rail_search_submit = Rechercher
rail_show_more = Afficher plus
rail_footer = Liens
chat_new = Nouvelle discussion
chat_search = Rechercher des conversations
chat_search_none = Aucune conversation ne correspond à votre recherche.
chat_start_title = Démarrer une conversation
chat_start_body = Choisissez une conversation dans la liste ou démarrez-en une nouvelle.
explore_sections = Sections Explorer
explore_see_more = Afficher plus
history_title = Historique
history_tabs = Sections de l’historique
history_likes_label = Posts aimés
video_play = Lecture
video_pause = Pause
video_mute = Couper le son
video_unmute = Activer le son
video_volume = Volume
video_seek = Position
video_position = { $current } sur { $total }
video_speed = Vitesse de lecture
video_fullscreen = Plein écran
video_exit_fullscreen = Quitter le plein écran
video_pip = Image dans l’image
video_exit_pip = Quitter l’image dans l’image
video_error = Cette vidéo ne peut pas être lue.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Masquer le panneau
lightbox_show_panel = Afficher le panneau
lightbox_view_post = Voir le post
lightbox_panel = Post et réponses
emotepick_tabs = Jeux d’émojis
emotepick_tab_emoji = Émojis
emotepick_tab_custom = Personnalisés
admin_emote_tab = Émotes
admin_emote_title = Émotes personnalisées
admin_emote_desc = Les émotes appartiennent à toute l’instance. Chacun peut les utiliser sous la forme :shortcode: dans les posts, les noms et les messages. Un shortcode ne peut pas être renommé ; supprimez l’émote et importez-la de nouveau.
admin_emote_shortcode = Shortcode
admin_emote_shortcode_hint = De 2 à 32 lettres, chiffres ou tirets bas. Affiché sous la forme :shortcode:.
admin_emote_invalid = Utilisez uniquement des lettres, des chiffres et des tirets bas, de 2 à 32 caractères.
admin_emote_category = Catégorie
admin_emote_category_hint = Facultatif. Regroupe les émotes dans le sélecteur.
admin_emote_file = Image de l’émote
admin_emote_file_hint = PNG, JPEG, WebP ou GIF, jusqu’à 256 Ko. Déposez un fichier ici ou choisissez-en un.
admin_emote_file_type = Ce fichier n’est pas une image PNG, JPEG, WebP ou GIF.
admin_emote_file_size = L’image est trop volumineuse (256 Ko max).
admin_emote_choose_file = Choisir une image
admin_emote_change_file = Changer l’image
admin_emote_upload = Importer l’émote
admin_emote_upload_failed = Impossible d’importer l’émote. Réessayez.
admin_emote_uploaded = :{ $shortcode }: ajouté.
admin_emote_list = Émotes de cette instance
admin_emote_none = Aucune émote personnalisée pour l’instant.
admin_emote_no_category = Aucune catégorie
admin_emote_category_for = Catégorie de :{ $shortcode }:
admin_emote_saved = Catégorie enregistrée.
admin_emote_delete = Supprimer :{ $shortcode }:
admin_emote_delete_title = Supprimer cette émote ?
admin_emote_delete_body = :{ $shortcode }: ne s’affichera plus comme une image nulle part et apparaîtra en texte brut. Les posts qui l’utilisaient conservent le texte.
admin_emote_delete_confirm = Supprimer l’émote
admin_emote_deleted = Émote supprimée.
admin_act_log_emoji_create = Émote ajoutée
admin_act_log_emoji_update = Émote modifiée
admin_act_log_emoji_delete = Émote supprimée
e2ee_settings = Messages chiffrés
e2ee_settings_desc = Phrase secrète, cet appareil et réinitialisation
e2ee_status_label = Statut
e2ee_status_none = Désactivé. Activez-le pour envoyer et lire des messages chiffrés de bout en bout.
e2ee_status_locked = Verrouillé sur cet appareil. Saisissez votre phrase secrète pour lire vos messages ici.
e2ee_status_ready = Activé et déverrouillé sur cet appareil.
e2ee_keys_error = Impossible de vérifier vos clés de chiffrement.
e2ee_setup_title = Activer les messages chiffrés
e2ee_setup_body = Les messages envoyés à des personnes de ce serveur sont chiffrés dans votre navigateur. Le serveur ne stocke que du texte qu’il ne peut pas lire. Une phrase secrète vous permet de déverrouiller vos messages sur un autre appareil.
e2ee_setup_warning = Votre phrase secrète ne peut pas être récupérée. Si vous l’oubliez, vous pouvez réinitialiser le chiffrement, mais vos anciens messages seront perdus.
e2ee_setup_submit = Activer
e2ee_setup_done = Les messages chiffrés sont activés
e2ee_unlock_title = Déverrouiller les messages chiffrés
e2ee_unlock_body = Saisissez votre phrase secrète pour lire et envoyer des messages chiffrés sur cet appareil.
e2ee_unlock_submit = Déverrouiller
e2ee_unlock_wrong = Cette phrase secrète est incorrecte.
e2ee_unlock_done = Déverrouillé
e2ee_passphrase = Phrase secrète
e2ee_passphrase_confirm = Confirmez la phrase secrète
e2ee_passphrase_hint = Au moins 10 caractères.
e2ee_passphrase_short = Utilisez au moins 10 caractères.
e2ee_passphrase_mismatch = Les phrases secrètes ne correspondent pas.
e2ee_failed = Une erreur s’est produite. Réessayez.
e2ee_unsupported = Ce navigateur ne prend pas en charge les messages chiffrés.
e2ee_forget = Oublier sur cet appareil
e2ee_forget_hint = Supprime de ce navigateur votre clé et les contacts auxquels vous faites confiance. Votre phrase secrète les déverrouille à nouveau.
e2ee_forget_done = Oublié sur cet appareil
e2ee_reset = Réinitialiser le chiffrement
e2ee_reset_hint = Crée une nouvelle clé. Les messages que vous avez envoyés et reçus auparavant deviennent illisibles.
e2ee_reset_title = Réinitialiser le chiffrement ?
e2ee_reset_body = Vos anciens messages deviennent illisibles, pour vous comme pour les personnes à qui vous avez écrit, et ne peuvent pas être récupérés. Vos contacts devront faire confiance à votre nouvelle clé. Choisissez une nouvelle phrase secrète.
e2ee_reset_submit = Réinitialiser
e2ee_reset_done = Le chiffrement a été réinitialisé
e2ee_note_sealed = Chiffré de bout en bout
e2ee_note_remote = Non chiffré de bout en bout : cette personne est sur un autre serveur.
e2ee_note_plain = Non chiffré de bout en bout.
e2ee_plain_mark = Non chiffré
e2ee_preview = Message chiffré
e2ee_line_sealed = Message chiffré
e2ee_line_own_key = Envoyé vers une clé que vous n’avez plus
e2ee_line_peer_key = Envoyé avec une clé à laquelle vous ne faites pas encore confiance
e2ee_line_unreadable = Impossible à ouvrir
e2ee_line_unsupported = Ce navigateur ne peut pas ouvrir les messages chiffrés
e2ee_block_peer_off = { $handle } n’a pas encore activé les messages chiffrés.
e2ee_block_self_off = Activez d’abord les messages chiffrés dans les Paramètres.
e2ee_block_unlock = Déverrouillez d’abord les messages chiffrés dans les Paramètres.
e2ee_block_downgrade = { $handle } a déjà utilisé les messages chiffrés. Cette conversation ne reviendra pas aux messages non chiffrés.
e2ee_block_changed = Faites confiance à la nouvelle clé de { $handle } pour lui écrire.
e2ee_block_error = Impossible de vérifier vos clés de chiffrement.
e2ee_changed_title = La clé de chiffrement de { $name } a changé
e2ee_changed_body = Cette personne a peut-être réinitialisé le chiffrement ou changé d’appareil, ou quelqu’un s’interpose. Comparez le nouveau numéro de sécurité avec elle avant de lui faire confiance. Vous ne pouvez pas envoyer de message tant que vous ne l’avez pas fait.
e2ee_changed_old = Numéro de sécurité auquel vous faisiez confiance
e2ee_changed_new = Nouveau numéro de sécurité
e2ee_trust = Faire confiance à la nouvelle clé
e2ee_safety_open = Numéro de sécurité
e2ee_safety_title = Numéro de sécurité
e2ee_safety_body = Comparez ce numéro avec { $name } en personne ou lors d’un appel, pas dans cette conversation. S’il est identique sur les deux écrans, personne ne s’interpose.

## 0031
composer_upload_failed = Échec de l’envoi. Réessayez.
composer_send_failed = Impossible de joindre tuita. Vérifiez votre connexion et réessayez.
composer_open_failed = Impossible d’ouvrir l’éditeur de publication. Réessayez.
