app_name = tuita
loading = Cargando
close = Cerrar
dismiss = Descartar
notifications = Notificaciones
avatar_more = { $count } más
char_counter_left =
    { $count ->
        [one] Queda { $count } carácter
       *[other] Quedan { $count } caracteres
    }
char_counter_over =
    { $count ->
        [one] { $count } carácter por encima del límite
       *[other] { $count } caracteres por encima del límite
    }
not_found_title = Aquí no hay nada
not_found_body = Esta página no existe o se movió.
not_found_back = Volver
tagline = Cada publicación, dos tintas.
error_title = Algo salió mal
cancel = Cancelar
retry = Reintentar
back = Atrás
view = Ver
follow = Seguir
following = Siguiendo
follows_you = Te sigue
verified = Cuenta verificada
bot_account = Cuenta automatizada
bot_label = Bot
link_copied = Enlace copiado al portapapeles
text_copied = Texto copiado
time_now = ahora
time_minutes = { $n }m
time_hours = { $n }h
time_days = { $n }d
nav_main = Principal
nav_home = Inicio
nav_explore = Explorar
nav_notifications = Notificaciones
nav_bookmarks = Guardados
nav_profile = Perfil
nav_settings = Ajustes
nav_compose = Publicar
nav_menu = Menú
badge_unread = { $count } sin leer
account_menu = Cuentas
account_switch = Cambiar de cuenta
account_switch_to = Cambiar a { $handle }
account_add = Añadir una cuenta
account_logout = Cerrar sesión de { $handle }
account_logout_all = Cerrar sesión en todas las cuentas
home_feed = Cronología
home_for_you = Para ti
home_following = Siguiendo
home_empty_title = Aún no hay nada aquí
home_empty_body = Sé el primero en publicar algo.
home_following_empty_title = Tu círculo está tranquilo
home_following_empty_body = Sigue a personas para llenar esta cronología con sus publicaciones y reposts.
home_find_people = Encontrar gente
new_posts =
    { $count ->
        [one] Mostrar { $count } publicación nueva
       *[other] Mostrar { $count } publicaciones nuevas
    }
new_posts_many = Mostrar más de 99 publicaciones nuevas
composer_title = Nueva publicación
composer_reply_title = Responder
composer_quote_title = Citar publicación
composer_placeholder = ¿Qué está pasando?
composer_reply_placeholder = Publica tu respuesta
composer_text_label = Texto de la publicación
composer_replying_to = Respondiendo a { $handle }
composer_cw_label = Advertencia de contenido
composer_cw_placeholder = Advertencia de contenido, p. ej. spoilers
composer_cw_toggle = Añadir una advertencia de contenido
composer_sensitive_toggle = Marcar multimedia como sensible
composer_options = Opciones de la publicación
composer_policy_label = Quién puede responder
composer_suggestions = Sugerencias
composer_tag_posts =
    { $count ->
        [one] { $compact } publicación
       *[other] { $compact } publicaciones
    }
composer_post = Publicar
composer_reply = Responder
policy_everyone = Todos pueden responder
policy_following = Pueden responder las personas que sigues
policy_mentioned = Solo pueden responder las personas que mencionas
post_sent = Publicación enviada
post_by = Publicación de { $name }
post_reposted_by = { $name } reposteó
post_pinned = Publicación fijada
post_replying_to = Respondiendo a { $handle }
post_replying_unavailable = Respondiendo a una publicación no disponible
post_unavailable = Esta publicación no está disponible.
post_menu = Más
post_copy_link = Copiar enlace
post_copy_text = Copiar texto
post_pin = Fijar en el perfil
post_unpin = Desfijar del perfil
post_pinned_toast = Fijado en tu perfil
post_unpinned = Desfijado de tu perfil
post_delete = Eliminar publicación
post_delete_title = ¿Eliminar publicación?
post_delete_body = No se puede deshacer. Se elimina de tu perfil, de las cronologías de quienes te siguen y de la búsqueda.
post_deleted = Publicación eliminada
post_view_profile = Ver a { $handle }
post_mute = Silenciar a { $handle }
post_block = Bloquear a { $handle }
post_reply_label = Responder ({ $count })
post_reply_following_only = Solo pueden responder las personas a las que sigue el autor.
post_reply_mentioned_only = Solo pueden responder las personas mencionadas en esta publicación.
post_repost_menu = Opciones de repost
post_repost = Repostear
post_undo_repost = Deshacer repost
post_quote = Citar
post_reposts_label = Reposts y citas ({ $count })
post_like_label = Me gusta ({ $count })
post_bookmark_label = Guardar
post_share = Compartir o copiar enlace
cw_show = Mostrar
cw_hide = Ocultar
cw_label = Advertencia de contenido: { $reason }
muted_toast = Silenciaste a { $handle }
unmuted_toast = Dejaste de silenciar a { $handle }
blocked_toast = Bloqueaste a { $handle }
unblocked_toast = Desbloqueaste a { $handle }
block_title = ¿Bloquear a { $handle }?
block_body = No podrá seguirte y no verás sus publicaciones. No se le avisa.
block_confirm = Bloquear
notify_like = A @{ $handle } le gustó tu publicación
notify_repost = @{ $handle } reposteó tu publicación
notify_reply = @{ $handle } te respondió
notify_mention = @{ $handle } te mencionó
notify_quote = @{ $handle } citó tu publicación
notify_follow = @{ $handle } empezó a seguirte
notify_follow_request = @{ $handle } solicitó seguirte
notify_poll = Terminó una encuesta en la que votaste
notify_affiliation = @{ $handle } te invitó a afiliarte
shortcuts_title = Atajos de teclado
shortcut_group_navigation = Navegación
shortcut_group_actions = Acciones
shortcut_or = /
shortcut_then = luego
shortcut_plus = +
shortcut_next_prev = Publicación siguiente / anterior
shortcut_open = Abrir publicación
shortcut_search = Buscar
shortcut_go_home = Inicio
shortcut_go_explore = Explorar
shortcut_go_notifications = Notificaciones
shortcut_go_bookmarks = Guardados
shortcut_go_profile = Tu perfil
shortcut_go_settings = Ajustes
shortcut_new_post = Nueva publicación
shortcut_like = Me gusta
shortcut_reply = Responder
shortcut_repost = Repostear o citar
shortcut_bookmark = Guardar
shortcut_send = Enviar publicación
shortcut_help = Mostrar esta ayuda
auth_login_title = Iniciar sesión
auth_add_title = Añadir otra cuenta
auth_add_sub = Inicia sesión para alternar entre hasta { $max } cuentas en este dispositivo sin cerrar sesión.
auth_signup_title = Crear cuenta
auth_handle = Usuario
auth_password = Contraseña
auth_display_name = Nombre visible
auth_handle_hint = 3–15 letras, números o guiones bajos
auth_password_hint = Al menos 8 caracteres
auth_consent_pre = Tengo al menos { $minAge } años y acepto los{" "}
auth_consent_terms = Términos del servicio
auth_consent_and = {" "}y la{" "}
auth_consent_privacy = Política de privacidad
auth_consent_end = {"."}
auth_consent_required = Acepta los Términos del servicio y la Política de privacidad.
auth_signin = Iniciar sesión
auth_signin_add = Iniciar sesión y cambiar
auth_create = Crear cuenta
auth_to_signup = ¿Eres nuevo? Crea una cuenta
auth_to_login = ¿Ya tienes cuenta? Inicia sesión
auth_closed = Los registros están cerrados en esta instancia por ahora.
landing_nav = Pie de página
landing_rules = Reglas de la casa
landing_stats = Ahora mismo
landing_people = Personas
landing_posts = Publicaciones
landing_clock = Tu hora local
landing_trends = Tendencias
landing_trend =
    { $count ->
        [one] { $count } publicación
       *[other] { $count } publicaciones
    }
landing_join = Únete a { $name }
landing_join_sub = Crea una cuenta y publica en dos tintas.
landing_closed = Los registros están cerrados por ahora. ¿Ya tienes cuenta? Inicia sesión.
landing_terms = Términos
landing_privacy = Privacidad
landing_source = Código fuente
landing_powered = Con tecnología de tuita { $version }
docs_terms_title = Términos del servicio
docs_privacy_title = Política de privacidad
docs_updated = Última actualización: { $date }
docs_date = 5 de noviembre de 2026
docs_see_privacy = Consulta también la Política de privacidad.
docs_see_terms = Consulta también los Términos del servicio.
docs_contact_fallback = los administradores de la instancia (por mensaje directo o mediante una denuncia)
docs_terms_agreement_h = 1. Acuerdo
docs_terms_agreement_p = Estos Términos del servicio («Términos») regulan tu uso de { $name }, la instancia de tuita en { $domain } («el servicio», «nosotros»). Al crear una cuenta o usar el servicio aceptas estos Términos y nuestra Política de privacidad. Si no estás de acuerdo, no uses el servicio.
docs_terms_eligibility_h = 2. Quién puede usarlo
docs_terms_eligibility_p = Debes tener al menos { $minAge } años, o la edad mínima de consentimiento digital de tu país si es mayor. Debes poder celebrar un acuerdo vinculante y no tener prohibido usar el servicio según la ley aplicable.
docs_terms_account_h = 3. Tu cuenta
docs_terms_account_1 = Mantén tu contraseña en secreto. Eres responsable de la actividad de tu cuenta.
docs_terms_account_2 = Tu usuario no debe suplantar a otra persona u organización ni engañar sobre quién eres.
docs_terms_account_3 = Avísanos pronto si crees que tu cuenta ha sido vulnerada. Puedes cerrar otras sesiones desde Ajustes → Cuenta.
docs_terms_content_h = 4. Tu contenido
docs_terms_content_p1 = Conservas la propiedad de lo que publicas. Nos otorgas una licencia mundial, no exclusiva y libre de regalías para almacenar, mostrar, reproducir y distribuir tu contenido únicamente para operar y mejorar el servicio. Eso incluye enviarlo a instancias federadas de tuita (consulta la sección 6). Esta licencia termina cuando eliminas el contenido o tu cuenta, salvo por las copias ya entregadas a otras instancias o guardadas por otras personas.
docs_terms_content_p2 = Eres responsable de lo que publicas. Publica solo contenido que tengas derecho a compartir.
docs_terms_rules_h = 5. Reglas de la comunidad
docs_terms_rules_lead = Al usar { $name } aceptas no infringir la ley ni estas reglas:
docs_terms_rules_p = También aceptas no atacar el servicio, por ejemplo intentando eludir medidas de seguridad, extrayendo datos a gran escala, enviando malware o sobrecargándolo.
docs_terms_federation_h = 6. Federación
docs_terms_federation_p = { $name } puede conectarse («federarse») con otras instancias independientes de tuita que aprueben los administradores. Cuando interactúas con personas de otra instancia, tu perfil público, tus publicaciones públicas y la interacción (por ejemplo un seguimiento, me gusta, repost, respuesta o mensaje directo) se envían a esa instancia. Las otras instancias las gestionan otras personas con sus propios términos. No podemos controlar ni eliminar el contenido una vez que otra instancia lo tiene, aunque sí les enviamos solicitudes de eliminación cuando borras algo.
docs_terms_moderation_h = 7. Moderación y cumplimiento
docs_terms_moderation_p = Los moderadores pueden eliminar contenido, limitar funciones o suspender cuentas que incumplan estos Términos o la ley. Cuando sea posible, te explicaremos el motivo. Cualquiera puede denunciar una publicación o una cuenta. Las denuncias llegan a los moderadores de esta instancia. Podemos dejar de federarnos con cualquier instancia en cualquier momento.
docs_terms_termination_h = 8. Salir del servicio
docs_terms_termination_p = Puedes exportar tus datos o eliminar tu cuenta en cualquier momento desde Ajustes → Cuenta. Podemos suspender o terminar tu acceso si incumples estos Términos de forma grave o repetida. También podemos cerrar el servicio e intentaremos avisar con antelación razonable.
docs_terms_disclaimers_h = 9. Exención de garantías y responsabilidad
docs_terms_disclaimers_p = El servicio se ofrece «tal cual» y «según disponibilidad», sin garantías de ningún tipo, en la medida máxima que permita la ley. No somos responsables de daños indirectos, incidentales o consecuentes, ni del contenido publicado por usuarios u otras instancias. Nada en estos Términos limita la responsabilidad que la ley no permite limitar, ni tus derechos legales como consumidor.
docs_terms_changes_h = 10. Cambios
docs_terms_changes_p = Podemos actualizar estos Términos. Si el cambio es importante, avisaremos en la aplicación antes de que entre en vigor. Seguir usando el servicio después implica que aceptas los nuevos Términos.
docs_terms_contact_h = 11. Contacto
docs_terms_contact_lead = Preguntas sobre estos Términos:
docs_privacy_summary_h = La versión corta
docs_privacy_summary_1 = Sin anuncios, sin scripts de rastreo y sin venta de datos.
docs_privacy_summary_2 = Una sola cookie, para mantener tu sesión iniciada. Las preferencias de tema se quedan en tu navegador.
docs_privacy_summary_3 = Las publicaciones públicas son públicas y pueden llegar a instancias federadas de tuita.
docs_privacy_summary_4 = Los mensajes directos entre cuentas de este servidor tienen cifrado de extremo a extremo cuando ambas personas lo activan: se almacenan como texto que el servidor no puede leer, así que una filtración de la base de datos o un vistazo a los datos almacenados no revela nada. Los mensajes a personas de otros servidores, y los enviados antes de activar el cifrado, no se cifran de este modo y los administradores pueden leerlos técnicamente. Este servidor también entrega la aplicación que cifra, así que un administrador que la modificara podría capturar lo que escribes.
docs_privacy_summary_5 = Puedes exportar o eliminar tus datos cuando quieras.
docs_privacy_controller_h = 1. Quiénes somos
docs_privacy_controller_p = Esta política cubre { $name } en { $domain }, gestionada por los administradores de su instancia. Ellos deciden cómo se tratan tus datos.
docs_privacy_collect_h = 2. Qué recopilamos
docs_privacy_collect_1 = Datos de la cuenta: usuario, nombre visible, contraseña (guardada solo como hash argon2id con sal) y los detalles de tu perfil: biografía, ubicación, sitio web, avatar, banner, forma y color.
docs_privacy_collect_2 = Contenido: publicaciones, imágenes, respuestas, me gusta, reposts, guardados, seguimientos, bloqueos, silenciados y denuncias.
docs_privacy_collect_3 = Mensajes directos: se almacenan en este servidor para poder entregártelos a ti y a la otra persona. Un mensaje con cifrado de extremo a extremo se almacena como texto cifrado que no podemos leer; quién lo envió, a quién y cuándo sigue siendo visible para nosotros. Los demás mensajes se almacenan como texto sin cifrar. Tu clave de cifrado se guarda aquí solo en una forma bloqueada por tu frase de contraseña, que nunca vemos.
docs_privacy_collect_4 = Datos de sesión: de cada dispositivo con sesión iniciada, el user-agent del navegador, la dirección IP y la hora del último uso, para que puedas revisar y cerrar sesiones.
docs_privacy_collect_5 = Datos de seguridad de corta duración: direcciones IP guardadas en memoria unos minutos para limitar la frecuencia de solicitudes.
docs_privacy_collect_p = No recopilamos tu correo, teléfono, contactos ni ubicación, salvo que los escribas en tu perfil.
docs_privacy_use_h = 3. Cómo los usamos
docs_privacy_use_p = Solo para operar el servicio: mostrar tu perfil y publicaciones, generar cronologías y notificaciones, entregar mensajes, mantener las cuentas seguras, prevenir abusos y hacer cumplir los Términos. La cronología «Para ti» ordena las publicaciones por me gusta, reposts, respuestas y antigüedad. No crea un perfil sobre ti.
docs_privacy_visibility_h = 4. Qué pueden ver los demás
docs_privacy_visibility_p = Tu perfil, publicaciones, respuestas, reposts, me gusta, seguidores y seguidos son públicos para cualquiera con sesión iniciada en esta instancia y para las instancias federadas. Cuando alguien comparte un enlace a una publicación o perfil, aplicaciones como Discord, Slack o iMessage pueden obtener una vista previa con la publicación pública, sus imágenes y los datos públicos de tu perfil. Las publicaciones con advertencia de contenido o marcadas como sensibles se previsualizan sin su texto ni imágenes. Los guardados, bloqueos, silenciados, denuncias y ajustes son privados. Si desactivas «Mostrarme en sugerencias», las personas aún pueden encontrarte por tu usuario exacto.
docs_privacy_federation_h = 5. Federación
docs_privacy_federation_p = Cuando sigues, respondes, mencionas, das me gusta, haces repost o envías un mensaje a alguien de otra instancia de tuita, enviamos a esa instancia los datos necesarios: tu perfil público, la publicación o interacción y el texto del mensaje en el caso de los mensajes directos. Los mensajes directos a otras instancias no tienen cifrado de extremo a extremo. Las instancias de tus seguidores reciben tus publicaciones públicas y las actualizaciones de tu perfil. Esas instancias funcionan de forma independiente y guardan los datos según sus propias políticas. Cuando eliminas una publicación o tu cuenta pedimos a las instancias federadas que borren sus copias, pero no podemos garantizar que lo hagan.
docs_privacy_cookies_h = 6. Cookies y almacenamiento local
docs_privacy_cookies_p = Usamos una única cookie de sesión estrictamente necesaria y HttpOnly. Tus preferencias de tema, color, tamaño de texto y movimiento se guardan en el almacenamiento local de tu navegador y nunca se nos envían.
docs_privacy_retention_h = 7. Cuánto tiempo conservamos los datos
docs_privacy_retention_1 = El contenido y los datos de la cuenta se conservan hasta que los eliminas.
docs_privacy_retention_2 = Las publicaciones eliminadas dejan de ser públicas de inmediato y se borran su texto e imágenes.
docs_privacy_retention_3 = Cuando eliminas tu cuenta borramos tus publicaciones, me gusta, seguimientos, mensajes y sesiones, y anonimizamos el registro de la cuenta.
docs_privacy_retention_4 = Las sesiones caducan a los 30 días, o de inmediato al cerrar sesión.
docs_privacy_retention_5 = Las denuncias de moderación y el registro de auditoría de administración se conservan el tiempo necesario para mantener el servicio seguro.
docs_privacy_rights_h = 8. Tus derechos
docs_privacy_rights_p = Según dónde vivas (por ejemplo bajo el RGPD o la CCPA) puedes tener derecho a acceder a tus datos, corregirlos, exportarlos o eliminarlos, y a oponerte al tratamiento o restringirlo. Puedes hacer la mayor parte tú mismo en Ajustes → Cuenta: editar tu perfil, descargar tus datos y eliminar tu cuenta. Para cualquier otra cosa, contacta con:
docs_privacy_rights_end = También puedes presentar una queja ante tu autoridad local de protección de datos.
docs_privacy_security_h = 9. Seguridad
docs_privacy_security_p = Las contraseñas se protegen con hash scrypt. Los tokens de sesión se guardan solo como hashes. Las solicitudes entre instancias federadas se firman con claves Ed25519. Ningún sistema es perfectamente seguro, así que usa una contraseña única.
docs_privacy_children_h = 10. Menores
docs_privacy_children_p = El servicio no está pensado para menores de la edad mínima indicada en los Términos. Si crees que un menor tiene una cuenta, denúncialo.
docs_privacy_changes_h = 11. Cambios
docs_privacy_changes_p = Publicaremos los cambios de esta política aquí y actualizaremos la fecha de arriba. Los cambios importantes se anunciarán en la aplicación.
docs_privacy_contact_h = 12. Contacto
docs_privacy_contact_lead = Preguntas o solicitudes sobre privacidad:
profile_posts =
    { $count ->
        [one] { $compact } publicación
       *[other] { $compact } publicaciones
    }
profile_edit = Editar perfil
profile_more = Más
profile_copy_link = Copiar enlace del perfil
profile_mute = Silenciar a { $handle }
profile_unmute = Dejar de silenciar a { $handle }
profile_block = Bloquear a { $handle }
profile_unblock = Desbloquear a { $handle }
profile_unblock_button = Desbloquear
profile_suspended = Esta cuenta está suspendida. Solo el equipo puede verla.
profile_muted = Silenciado
profile_org = Organización
profile_role_admin = Administración
profile_role_moderator = Mod
profile_joined = Se unió en { $date }
profile_following_label = Siguiendo
profile_followers =
    { $count ->
        [one] Seguidor
       *[other] Seguidores
    }
profile_counts = Cifras de seguimiento
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Seguido por { $a }
            }
        [2]
            { $others ->
               *[other] Seguido por { $a } y { $b }
            }
       *[other]
            { $others ->
                [one] Seguido por { $a }, { $b } y { $others } más que sigues
               *[other] Seguido por { $a }, { $b } y { $others } más que sigues
            }
    }
profile_banner = Banner del perfil
profile_tabs = Secciones del perfil
profile_tab_posts = Publicaciones
profile_tab_replies = Respuestas
profile_tab_media = Multimedia
profile_tab_likes = Me gusta
profile_posts_label = Publicaciones
profile_empty_posts_self_title = Aún no has publicado
profile_empty_posts_self_body = Aquí aparecerán tus publicaciones y reposts.
profile_empty_posts_title = { $handle } no ha publicado
profile_empty_posts_body = Cuando lo haga, sus publicaciones aparecerán aquí.
profile_empty_replies_title = Aún no hay respuestas
profile_empty_replies_body = Aquí aparecerán las respuestas.
profile_empty_media_title = Aún no hay fotos
profile_empty_media_body = Aquí aparecerán las publicaciones con imágenes.
profile_empty_likes_title = Aún no hay me gusta
profile_empty_likes_body = Aquí aparecerán las publicaciones que te gustan.
profile_blocked_title = Bloqueaste a { $handle }
profile_blocked_body = No verás sus publicaciones y no podrá seguirte ni escribirte.
profile_hidden_title = No puedes ver estas publicaciones
profile_hidden_body = { $handle } ha limitado quién puede ver su perfil.
follows_tabs = Listas de seguimiento
follows_tab_followers = Seguidores
follows_tab_following = Siguiendo
follows_label_followers = Seguidores
follows_label_following = Siguiendo
follows_empty_followers_title = Aún no tiene seguidores
follows_empty_followers_body = Cuando alguien siga esta cuenta, aparecerá aquí.
follows_empty_following_title = No sigue a nadie
follows_empty_following_body = Las cuentas que siga aparecerán aquí.
thread_title = Publicación
thread_doc_title = { $name }: «{ $text }»
thread_views =
    { $count ->
        [one] Visualización
       *[other] Visualizaciones
    }
thread_reposts =
    { $count ->
        [one] Repost
       *[other] Reposts
    }
thread_quotes =
    { $count ->
        [one] Cita
       *[other] Citas
    }
thread_likes =
    { $count ->
        [one] Me gusta
       *[other] Me gusta
    }
thread_stats = Estadísticas de la publicación
thread_replies = Respuestas
thread_hidden = Respuestas ocultas ({ $count })
thread_hidden_note = El autor ocultó estas respuestas.
thread_list_likes = Les gusta a
thread_list_reposts = Reposteado por
thread_list_quotes = Citas
thread_list_empty = Nadie todavía
tag_empty_title = No hay publicaciones con { $tag }
tag_empty_body = Sé el primero en usarlo.
tag_posts_label = Publicaciones con { $tag }
bookmarks_title = Guardados
bookmarks_label = Publicaciones guardadas
bookmarks_empty_title = Guarda publicaciones para después
bookmarks_empty_body = Toca el icono de guardar en cualquier publicación para tenerla aquí. Solo tú ves tus guardados.
settings_account = Tu cuenta
settings_account_desc = Perfil y contraseña
settings_profile = Editar perfil
settings_profile_desc = Nombre, biografía, forma del avatar y tinta
settings_privacy = Privacidad y seguridad
settings_privacy_desc = Mensajes, visibilidad, palabras silenciadas, cuentas bloqueadas y silenciadas
settings_notifications = Notificaciones
settings_notifications_desc = Elige de qué quieres recibir notificaciones
settings_appearance = Apariencia
settings_appearance_desc = Tema, tinta, tamaño del texto, contraste y movimiento
settings_language = Idioma
settings_language_desc = Idioma de la interfaz
settings_language_note = Se guarda en este navegador y en tu cuenta si tienes la sesión iniciada.
settings_about = Acerca de esta instancia
settings_about_desc = Reglas, términos, privacidad, versión y atajos de teclado
settings_logout = Cerrar sesión
settings_account_profile = Perfil
settings_account_joined = Se unió en { $date }
settings_account_edit = Editar perfil
settings_password_title = Cambiar contraseña
settings_password_current = Contraseña actual
settings_password_new = Nueva contraseña
settings_password_hint = Al menos 8 caracteres. Se cerrará la sesión en otros dispositivos.
settings_password_confirm = Confirma la nueva contraseña
settings_password_submit = Actualizar contraseña
settings_password_mismatch = Las nuevas contraseñas no coinciden.
settings_password_updated = Contraseña actualizada. Se cerraron las demás sesiones.
settings_sessions_title = Dispositivos y sesiones
settings_sessions_desc = Cierra cualquier sesión que no reconozcas y luego cambia tu contraseña.
settings_device_on = { $browser } en { $os }
settings_device_unknown = Dispositivo desconocido
settings_session_current = Este dispositivo
settings_session_last_active = Última actividad: { $time }
settings_session_sign_out = Cerrar sesión
settings_session_sign_out_device = Cerrar sesión en { $device }
settings_session_signed_out = Sesión cerrada
settings_sessions_revoke_all = Cerrar sesión en todos los demás dispositivos
settings_sessions_revoke_title = ¿Cerrar sesión en otros dispositivos?
settings_sessions_revoke_body = Se cerrará la sesión en todos los dispositivos excepto en este.
settings_sessions_revoke_confirm = Cerrar las demás
settings_sessions_revoked =
    { $count ->
        [one] { $count } sesión cerrada
       *[other] { $count } sesiones cerradas
    }
settings_profile_preview = Vista previa
settings_profile_preview_alt = Vista previa del avatar
settings_profile_uploads = La subida de avatar y banner llegará más adelante. Mientras tanto se usan tu avatar y banner generados.
settings_profile_details = Detalles
settings_profile_name = Nombre visible
settings_profile_bio = Biografía
settings_profile_location = Ubicación
settings_profile_website = Sitio web
settings_profile_shape = Forma del avatar
settings_profile_ink = Tinta del perfil
settings_profile_save = Guardar perfil
settings_profile_saved = Perfil actualizado
settings_shape_square = Cuadrado
settings_shape_circle = Círculo
settings_shape_squircle = Superelipse
settings_shape_octagon = Octágono
settings_ink_bone = Hueso
settings_ink_phosphor = Fósforo
settings_ink_amber = Ámbar
settings_ink_cyan = Cian
settings_ink_rose = Rosa
settings_dm_title = Mensajes directos
settings_dm_desc = Quién puede iniciar una conversación contigo
settings_dm_label = Quién puede escribirte
settings_dm_everyone = Todos
settings_dm_following = Personas que sigo
settings_dm_none = Nadie
settings_dm_saved = Ajustes de mensajes guardados
settings_discover_title = Visibilidad
settings_discoverable = Mostrarme en sugerencias y búsquedas
settings_discoverable_hint = Aún pueden encontrarte por tu usuario exacto
settings_show_client = Mostrar desde qué dispositivo publico
settings_show_client_hint = Añade el dispositivo bajo tus publicaciones nuevas
settings_words_title = Palabras silenciadas
settings_words_desc = Oculta de tus cronologías, búsquedas y respuestas las publicaciones con estas palabras, frases o #hashtags. Tus propias publicaciones nunca se ocultan.
settings_words_label = Palabra para silenciar
settings_words_placeholder = Añade una palabra, frase o #hashtag
settings_words_add = Silenciar
settings_words_remove = Dejar de silenciar { $word }
settings_words_none = No hay palabras silenciadas.
settings_words_limit = Puedes silenciar hasta { $count } palabras.
settings_blocked_title = Cuentas bloqueadas
settings_blocked_desc = No pueden seguirte, escribirte ni ver tus publicaciones o interactuar con ellas. No se les avisa.
settings_blocked_empty = No has bloqueado a nadie.
settings_muted_title = Cuentas silenciadas
settings_muted_desc = No verás sus publicaciones ni notificaciones. Aún podrán seguirte y escribirte.
settings_muted_empty = No has silenciado a nadie.
settings_unblock = Desbloquear
settings_unmute = Dejar de silenciar
settings_unblock_user = Desbloquear a { $handle }
settings_unmute_user = Dejar de silenciar a { $handle }
settings_notif_title = Avisarme de
settings_notif_desc = Las cuentas silenciadas y bloqueadas nunca te notifican.
settings_notif_like = Me gusta
settings_notif_like_hint = Cuando a alguien le gusta tu publicación
settings_notif_repost = Reposts
settings_notif_repost_hint = Cuando alguien repostea tu publicación
settings_notif_reply = Respuestas
settings_notif_reply_hint = Cuando alguien te responde
settings_notif_mention = Menciones
settings_notif_mention_hint = Cuando alguien te menciona
settings_notif_quote = Citas
settings_notif_quote_hint = Cuando alguien cita tu publicación
settings_notif_follow = Nuevos seguidores
settings_notif_follow_hint = Cuando alguien te sigue
settings_desktop_title = Notificaciones de escritorio
settings_desktop_desc = Se muestra como notificación del sistema solo mientras esta pestaña está en segundo plano.
settings_desktop_state = Permiso: { $state }
settings_desktop_allowed = Permitido
settings_desktop_blocked = Bloqueado
settings_desktop_not_asked = No solicitado
settings_desktop_enable = Activar
settings_desktop_blocked_toast = Las notificaciones de este sitio están bloqueadas en los ajustes del navegador.
settings_desktop_unsupported = Este navegador no admite notificaciones de escritorio.
settings_theme_title = Tema
settings_appearance_note = Se guarda en este dispositivo y se aplica al instante.
settings_theme_mode = Tema
settings_theme_system = Sistema
settings_theme_light = Claro
settings_theme_dark = Oscuro
settings_theme_ink = Tinta
settings_theme_contrast = Contraste
settings_contrast_normal = Normal
settings_contrast_more = Más
settings_reading_title = Texto y movimiento
settings_theme_text_size = Tamaño del texto
settings_theme_motion = Movimiento
settings_motion_system = Sistema
settings_motion_reduce = Reducir
settings_preview_title = Vista previa
settings_preview_text = Así se leerán las publicaciones con tus ajustes.
settings_about_domain = Dominio
settings_about_version = Versión
settings_about_contact = Contacto
settings_about_terms = Términos del servicio
settings_about_privacy = Política de privacidad
settings_about_source = Código fuente
settings_about_rules = Reglas
explore_search_label = Buscar publicaciones, personas y hashtags
explore_search_placeholder = Busca publicaciones, personas, #hashtags
explore_clear = Borrar búsqueda
explore_tips = Consejos de búsqueda
explore_tips_title = Operadores de búsqueda
explore_op_from = Publicaciones de una cuenta
explore_op_to = Respuestas o menciones a una cuenta
explore_op_media = Publicaciones con imágenes (o has:poll, has:links)
explore_op_since = Publicado desde una fecha
explore_op_until = Publicado hasta una fecha
explore_op_phrase = Palabras en este orden
explore_op_exclude = Excluir publicaciones con una palabra
explore_results = Resultados de búsqueda
explore_tab_posts = Publicaciones
explore_tab_people = Personas
explore_tab_media = Multimedia
explore_people = Personas
explore_posts = Publicaciones
explore_recent = Búsquedas recientes
explore_recent_clear = Borrar
explore_trending = Tendencias
explore_trend_posts =
    { $count ->
        [one] { $compact } publicación
       *[other] { $compact } publicaciones
    }
explore_trends_empty = Aún no hay tendencias. ¡Usa un #hashtag!
explore_suggestions = A quién seguir
explore_suggestions_empty = Sigues a todo el mundo. Impresionante.
explore_no_posts_title = No hay publicaciones para «{ $q }»
explore_no_posts_body = Prueba con otra palabra o hashtag.
explore_no_people_title = No hay personas para «{ $q }»
explore_no_people_body = Prueba con un nombre, un usuario o @usuario@instancia para alguien de otra instancia.
explore_no_media_title = No hay imágenes para «{ $q }»
explore_no_media_body = Aquí aparecerán las publicaciones con imágenes que coincidan.
notif_filter = Filtro de notificaciones
notif_tab_all = Todas
notif_tab_mentions = Menciones
notif_empty_all_title = Aún no hay notificaciones
notif_empty_all_body = Aquí aparecerán los me gusta, respuestas, reposts y nuevos seguidores.
notif_empty_mentions_title = Aún no hay menciones
notif_empty_mentions_body = Aquí aparecen las respuestas, menciones y citas de tus publicaciones.
notif_who_two = { $first } y { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } y { $count } más
       *[other] { $first }, { $second } y { $count } más
    }
notif_verb_like =
    { $count ->
        [one] dio me gusta a tu publicación
       *[other] dieron me gusta a tu publicación
    }
notif_verb_repost =
    { $count ->
        [one] reposteó tu publicación
       *[other] repostearon tu publicación
    }
notif_verb_follow =
    { $count ->
        [one] te empezó a seguir
       *[other] te empezaron a seguir
    }
notif_verb_follow_request =
    { $count ->
        [one] solicitó seguirte
       *[other] solicitaron seguirte
    }
notif_verb_reply =
    { $count ->
        [one] te respondió
       *[other] te respondieron
    }
notif_verb_mention =
    { $count ->
        [one] te mencionó
       *[other] te mencionaron
    }
notif_verb_quote =
    { $count ->
        [one] citó tu publicación
       *[other] citaron tu publicación
    }
notif_verb_poll =
    { $count ->
        [one] terminó una encuesta en la que votaste
       *[other] terminaron una encuesta en la que votaste
    }
notif_verb_affiliation =
    { $count ->
        [one] te invitó a afiliarte
       *[other] te invitaron a afiliarte
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Programar
composer_post_all = Publicar todo
composer_err_type = Ese archivo no es una imagen o vídeo que tuita pueda usar (PNG, JPEG, GIF, WebP, MP4, WebM o MOV).
composer_err_image_size = La imagen es demasiado grande (máx. 10 MB).
composer_err_video_size = Los vídeos pueden ocupar hasta { $mb } MB.
composer_err_max_images = Puedes adjuntar hasta 4 imágenes.
composer_err_exclusive = Una publicación puede tener un vídeo, imágenes o una encuesta, pero solo una de esas cosas.
composer_extra_placeholder = Añadir otra publicación
composer_extra_label = Publicación { $n } del hilo
composer_extra_remove = Quitar publicación { $n } del hilo
composer_schedule_chip = Se publicará el { $time }
composer_schedule_clear = No programar
composer_add_images = Añadir imágenes
composer_add_video = Añadir vídeo
composer_add_poll = Añadir encuesta
composer_add_post = Añadir otra publicación
composer_scheduled = Programada para el { $time }
composer_thread_sent = Hilo de { $count } publicaciones enviado
composer_remove_video = Quitar vídeo
composer_remove_image = Quitar imagen
composer_uploading = Subiendo
composer_uploading_video = Subiendo vídeo… { $percent }%
composer_alt_video = Describe este vídeo
composer_alt_image = Describe esta imagen
composer_alt_label = Descripción (texto alternativo) del archivo adjunto { $index }
composer_emoji = Añadir emoji
poll_length_hours = { $n } h
poll_length_days = { $n } d
poll_choice = Opción { $n }
poll_choice_optional = Opción { $n } (opcional)
poll_remove_choice = Quitar opción { $n }
poll_add_choice = Añadir opción
poll_length = Duración de la encuesta
poll_multiple_label = Permitir varias opciones
poll_remove = Quitar encuesta
schedule_title = Programar publicación
schedule_hour = En una hora
schedule_evening = Esta noche
schedule_tomorrow = Mañana por la mañana
schedule_week = En una semana
schedule_window = Elige una hora entre un minuto y 90 días a partir de ahora.
schedule_custom = Elegir fecha y hora
schedule_set = Fijar
schedule_clear = No programar
schedule_view = Ver publicaciones programadas
dropzone_drop = Suelta para adjuntar
dropzone_reject = No se puede usar este archivo
nav_messages = Mensajes
nav_lists = Listas
nav_scheduled = Programadas
notify_message = @{ $handle } te envió un mensaje
shortcut_go_messages = Mensajes
shortcut_go_lists = Listas
cropper_stage = Área de recorte. Arrastra para mover la imagen. Usa las flechas para ajustarla y más y menos para el zoom.
cropper_hint = Arrastra para recolocar. Pellizca, desplaza o usa el control deslizante para hacer zoom.
cropper_zoom = Zoom
cropper_zoom_in = Acercar
cropper_zoom_out = Alejar
cropper_rotate = Girar
cropper_reset = Restablecer
cropper_apply = Aplicar
image_field_avatar = Foto de perfil
image_field_banner = Banner
image_field_edit_avatar = Editar foto de perfil
image_field_edit_banner = Editar banner
image_field_change = Cambiar
image_field_remove = Quitar
image_field_hint = PNG, JPEG, GIF o WebP de hasta 10 MB. Un GIF se convierte en imagen fija.
image_field_not_image = Ese archivo no es una imagen que tuita pueda usar (PNG, JPEG, GIF o WebP).
image_field_too_large = La imagen es demasiado grande (máx. 10 MB).
image_field_uploading = Subiendo
image_field_saved = Imagen guardada.
image_field_removed = Imagen eliminada.
edit_title = Editar publicación
edit_note = Puedes editar durante 30 minutos tras publicar. Todos pueden ver que una publicación se editó y sus versiones anteriores.
edit_save = Guardar
edit_saved = Publicación actualizada
edit_edited = Editada
edit_history_title = Historial de ediciones
edit_history_current = Versión actual
edit_history_earlier = Versión anterior
edit_history_original = Original
report_post_title = Denunciar publicación
report_user_title = Denunciar a { $handle }
report_intro = ¿Qué ocurre? Los moderadores de esta instancia revisarán tu denuncia. { $handle } no verá quién lo denunció.
report_reason = Motivo
report_note = ¿Algo más? (opcional)
report_send = Enviar denuncia
report_sent = Gracias. Tu denuncia se envió a los moderadores.
report_duplicate = Ya denunciaste esto. Los moderadores lo revisarán.
report_spam = Spam
report_spam_body = Anuncios no deseados, estafas, contenido repetitivo o engañoso
report_harassment = Acoso
report_harassment_body = Atacar, acosar o amenazar a alguien
report_hate = Odio
report_hate_body = Ataca a personas por quiénes son
report_violence = Violencia o autolesiones
report_violence_body = Amenazas, glorificar la violencia, incitar a autolesionarse
report_misinformation = Desinformación
report_misinformation_body = Afirmaciones falsas que podrían causar daño
report_impersonation = Suplantación
report_impersonation_body = Hacerse pasar por otra persona
report_other = Otra cosa
report_other_body = Incumple las reglas de otra forma
post_edit = Editar publicación
post_edit_history = Historial de ediciones
post_report = Denunciar publicación
post_hide_reply = Ocultar respuesta
post_unhide_reply = Mostrar respuesta
post_share_menu = Compartir…
post_reply_hidden = Respuesta oculta
post_reply_hidden_toast = Respuesta oculta
post_reply_unhidden_toast = Respuesta visible de nuevo
sensitive_label = Multimedia sensible
sensitive_show = Mostrar
sensitive_hide = Ocultar multimedia sensible
media_label =
    { $count ->
        [one] { $count } archivo adjunto
       *[other] { $count } archivos adjuntos
    }
media_video = Vídeo
media_open = Abrir imagen { $index } de { $count }
lightbox_title = Visor multimedia
lightbox_prev = Anterior
lightbox_next = Siguiente
poll_vote = Votar
poll_your_vote = Tu voto
poll_votes =
    { $count ->
        [one] { $compact } voto
       *[other] { $compact } votos
    }
poll_final = Resultados finales
poll_multiple = opción múltiple
poll_left_minutes = Quedan { $n } min
poll_left_hours = Quedan { $n } h
poll_left_days = Quedan { $n } d
pull_refresh = Desliza para actualizar
messages_title = Mensajes
messages_new = Nuevo mensaje
messages_label = Conversaciones
messages_empty_title = Inicia una conversación
messages_empty_body = Los mensajes que envíes y recibas aparecerán aquí.
messages_you_prefix = Tú: { $text }
messages_unread = { $count } sin leer
messages_new_search = Buscar personas
messages_new_results = Personas
messages_new_none = No se encontró a nadie.
chat_load_older = Cargar mensajes anteriores
chat_intro = Este es el inicio de tu conversación.
chat_label = Mensajes con { $name }
chat_delete = Eliminar
chat_seen = Visto
chat_delete_title = ¿Eliminar este mensaje?
chat_delete_body = Desaparece para los dos.
chat_doc_title = Chat con { $name }
chat_error_title = No se pudo abrir esta conversación
chat_placeholder = Escribe un mensaje
chat_input_label = Mensaje
chat_send = Enviar
lists_title = Listas
lists_new = Nueva lista
lists_edit = Editar lista
lists_label = Tus listas
lists_empty_title = Aún no hay listas
lists_empty_body = Agrupa personas en listas para leer sus publicaciones por separado.
lists_members =
    { $count ->
        [one] { $compact } miembro
       *[other] { $compact } miembros
    }
lists_private = Privada
lists_name = Nombre
lists_description = Descripción
lists_private_switch = Lista privada
lists_save = Guardar
lists_picker_title = Añadir a { $handle } a listas
lists_picker_none = Aún no tienes listas.
lists_delete = Eliminar lista
lists_delete_title = ¿Eliminar esta lista?
lists_delete_body = Se eliminará «{ $name }». Las personas que contiene no se ven afectadas.
lists_by = Lista de { $handle }
lists_tab_posts = Publicaciones
lists_tab_members = Miembros
lists_tabs = Secciones de la lista
lists_error_title = No se pudo abrir esta lista
lists_posts_label = Publicaciones de la lista
lists_posts_empty_title = Aún no hay nada aquí
lists_posts_empty_body = Aquí aparecen las publicaciones de los miembros de esta lista.
lists_members_label = Miembros de la lista
lists_members_empty_title = Sin miembros
lists_members_empty_body = Añade personas desde sus perfiles.
lists_remove = Quitar
scheduled_title = Publicaciones programadas
scheduled_label = Publicaciones programadas
scheduled_status_pending = Programada
scheduled_status_published = Publicada
scheduled_status_failed = Fallida
scheduled_cw = Advertencia de contenido: { $text }
scheduled_poll = Encuesta con { $count } opciones, dura { $hours } h
scheduled_change_time = Cambiar hora
scheduled_post_now = Publicar ahora
scheduled_remove = Quitar
scheduled_window_error = Elige una hora entre un minuto y 90 días a partir de ahora.
scheduled_when = Publicar el
scheduled_zone = Zona horaria: { $zone }
scheduled_save = Guardar
scheduled_posted = Publicada.
scheduled_cancel_title = ¿Cancelar esta publicación programada?
scheduled_cancel_body = No se publicará.
scheduled_cancel_confirm = Descartar publicación
scheduled_empty_title = Nada programado
scheduled_empty_body = Las publicaciones que programes esperan aquí hasta su hora.
profile_message = Mensaje
profile_add_to_lists = Añadir a listas
media_alt = ALT
profile_report = Denunciar a { $handle }
chart_show_table = Mostrar { $title } como tabla
chart_col_day = Día
chart_col_hour = Hora (UTC)
chart_empty = { $title }: aún no hay nada que mostrar.
chart_bar_summary = { $title }: { $total } en total, el máximo el { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } en total, { $last } el { $label }.
chart_heat_summary = { $title }: { $total } en total, el momento de más actividad el { $label } a las { $hour } ({ $peak }).
chart_heat_less = Menos
chart_heat_more = Más
announce_region = Anuncios
announce_dismiss = Descartar anuncio
announce_more = Leer más
announce_tone_info = Aviso
announce_tone_success = Buenas noticias
announce_tone_warning = Advertencia
admin_stat_people = Personas
admin_stat_people_sub = { $count } activas esta semana
admin_stat_posts_today = Publicaciones, 24 h
admin_stat_posts_total = { $count } en total
admin_stat_reports = Denuncias abiertas
admin_stat_reports_open = Requiere revisión
admin_stat_reports_clear = Todo en orden
admin_stat_signups_today = Registros hoy
admin_stat_signups_sub = { $count } en 14 días
admin_stat_suspended = Suspendidas
admin_stat_suspended_sub = cuentas
admin_chart_posts = Publicaciones por día
admin_chart_signups = Registros por día
admin_chart_reports = Denuncias por día
admin_chart_heat = Actividad por día de la semana y hora
nav_admin = Administración
admin_title = Administración
admin_sections = Secciones de administración
admin_tab_overview = Resumen
admin_tab_reports = Denuncias
admin_tab_users = Personas
admin_tab_instance = Instancia
admin_tab_federation = Federación
admin_tab_announcements = Anuncios
admin_tab_invites = Invitaciones
admin_tab_filter = Filtro de contenido
admin_tab_audit = Registro de auditoría
admin_denied_title = Solo para el equipo
admin_denied_body = Esta área es para los administradores y moderadores de esta instancia.
admin_denied_home = Ir al inicio
admin_unknown = desconocido
admin_tag_active = Activa
admin_tag_suspended = Suspendida
admin_tag_deleted = Eliminada
admin_tag_org = Organización
admin_audit_system = sistema
admin_audit_empty = Aún no hay nada registrado
admin_report_status = Estado de la denuncia
admin_report_open = Abierta
admin_report_resolved = Resuelta
admin_reports_empty_open = No hay denuncias abiertas
admin_reports_empty_open_body = Cuando alguien denuncie publicaciones o cuentas, aparecerán aquí.
admin_reports_empty_resolved = Aún no hay nada resuelto
admin_report_by = denunciado por { $handle }
admin_report_same_target = { $count } denuncias abiertas
admin_report_media =
    { $count ->
        [one] { $count } archivo adjunto
       *[other] { $count } archivos adjuntos
    }
admin_report_poll = Tiene una encuesta
admin_report_dismiss = Descartar
admin_report_delete_post = Eliminar publicación
admin_report_suspend = Suspender cuenta
admin_report_delete_suspend = Eliminar y suspender
admin_report_confirm_title = ¿Tomar medidas?
admin_report_confirm_body = { $action }. Queda anotado en el registro de auditoría.
admin_report_confirm = Confirmar
admin_report_resolution = { $action } por { $handle } · { $time }
admin_res_dismiss = Desestimada
admin_res_delete_post = Publicación eliminada
admin_res_suspend_user = Cuenta suspendida
admin_res_delete_and_suspend = Publicación eliminada y cuenta suspendida
admin_users_search = Buscar personas
admin_users_search_placeholder = Nombre o usuario
admin_users_show = Mostrar
admin_users_empty = No se encontró a nadie
admin_filter_local = Locales
admin_filter_remote = Remotas
admin_filter_staff = Equipo
admin_filter_verified = Verificadas
admin_filter_suspended = Suspendidas
admin_filter_all = Todas
admin_user_details = Detalles de { $handle }
admin_user_menu = Gestionar a { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } publicación
       *[other] { $compact } publicaciones
    }
admin_user_followers =
    { $count ->
        [one] { $compact } seguidor
       *[other] { $compact } seguidores
    }
admin_user_joined = se unió { $time }
admin_user_active = activo { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } denuncia abierta
       *[other] { $compact } denuncias abiertas
    }
admin_act_verify = Verificar
admin_act_unverify = Quitar verificación
admin_act_org = Convertir en organización
admin_act_unorg = Quitar estado de organización
admin_act_role_user = Quitar rol de equipo
admin_act_role_moderator = Hacer moderador
admin_act_role_admin = Hacer administrador
admin_act_suspend = Suspender
admin_act_unsuspend = Quitar suspensión
admin_act_purge = Eliminar todas las publicaciones
admin_act_rename = Cambiar usuario
admin_act_reset_name = Restablecer nombre visible
admin_act_clear_bio = Borrar biografía
admin_act_delete = Eliminar cuenta
admin_done_verified = { $handle } está verificado
admin_done_unverified = Verificación eliminada
admin_done_org = { $handle } ahora es una organización
admin_done_unorg = Estado de organización eliminado
admin_done_role = Rol actualizado
admin_done_suspended = { $handle } suspendido
admin_done_unsuspended = Se quitó la suspensión de { $handle }
admin_done_purged =
    { $count ->
        [one] { $count } publicación eliminada
       *[other] { $count } publicaciones eliminadas
    }
admin_done_name_reset = Nombre visible restablecido
admin_done_bio_cleared = Biografía borrada
admin_done_deleted = { $handle } eliminado
admin_done_renamed = Ahora es { $handle }
admin_role_phrase_user = un usuario normal
admin_role_phrase_moderator = un moderador
admin_role_phrase_admin = un administrador
admin_role_title = ¿Cambiar el rol de { $handle }?
admin_role_body = Pasará a ser { $role }.
admin_role_confirm = Cambiar rol
admin_unorg_title = ¿Quitar el estado de organización a { $handle }?
admin_unorg_body = Todas sus afiliaciones terminan.
admin_unorg_confirm = Quitar
admin_suspend_title = ¿Suspender a { $handle }?
admin_suspend_body = Se le cierra la sesión y su contenido permanece oculto hasta que quites la suspensión.
admin_suspend_confirm = Suspender
admin_purge_title = ¿Eliminar todas las publicaciones de { $handle }?
admin_purge_body = No se puede deshacer.
admin_purge_confirm = Eliminar todo
admin_reset_name_title = ¿Restablecer el nombre visible de { $handle }?
admin_reset_name_body = Se sustituirá por su usuario.
admin_reset_name_confirm = Restablecer
admin_delete_title = ¿Eliminar a { $handle }?
admin_delete_body = Se eliminan sus publicaciones, seguimientos y mensajes, y la cuenta se anonimiza. No se puede deshacer.
admin_delete_confirm = Eliminar cuenta
admin_rename_title = Cambiar el usuario de { $handle }
admin_rename_note = Los enlaces al usuario anterior dejan de funcionar.
admin_rename_handle = Nuevo usuario
admin_rename_reset_name = Restablecer también el nombre visible
admin_rename_save = Cambiar usuario
admin_detail_title = Detalles de la cuenta
admin_detail_invite = Invitación { $code }
admin_detail_posts = Publicaciones
admin_detail_followers = Seguidores
admin_detail_following = Siguiendo
admin_detail_open_reports = Denuncias abiertas
admin_detail_sessions = Sesiones
admin_detail_passkeys = Llaves de acceso
admin_detail_tokens = Tokens de API
admin_detail_joined = Se unió en { $time }
admin_detail_recent_posts = Publicaciones recientes
admin_detail_no_posts = Sin publicaciones.
admin_detail_reports = Denuncias sobre esta cuenta
admin_detail_no_reports = Sin denuncias.
admin_detail_history = Historial de moderación
admin_detail_nothing = Aún no hay nada.
admin_instance_title = Instancia
admin_instance_name = Nombre de la instancia
admin_instance_description = Descripción
admin_instance_contact = Correo de contacto
admin_instance_contact_hint = Se muestra en las páginas de Términos y Privacidad
admin_instance_min_age = Edad mínima
admin_instance_rules = Reglas
admin_instance_rules_hint = Una regla por línea
admin_instance_preview = Vista previa de los términos
admin_instance_save = Guardar
admin_instance_saved = Ajustes de la instancia guardados
admin_instance_template_note = Los Términos del servicio y la Política de privacidad son plantillas que usan estos valores. Haz que los revisen según tu jurisdicción antes de abrir la instancia al público.
admin_signups_title = Registros
admin_signups_desc = Quién puede crear una cuenta en esta instancia
admin_signups_open = Abiertos
admin_signups_open_hint = Cualquiera puede crear una cuenta.
admin_signups_invite = Solo con invitación
admin_signups_invite_hint = Se necesita un código de invitación de la pestaña Invitaciones.
admin_signups_closed = Cerrados
admin_signups_closed_hint = Nadie puede registrarse.
admin_filter_title = Filtro de contenido
admin_filter_desc = No se bloquea nada hasta que añadas palabras abajo. Las palabras bloqueadas se aplican a usuarios, nombres visibles, ubicaciones, biografías, publicaciones y nombres de listas, incluidas las grafías disfrazadas. Los usuarios reservados como «admin» no se pueden registrar.
admin_filter_blocked = Palabras bloqueadas
admin_filter_blocked_hint = Una por línea, se buscan dentro de las palabras
admin_filter_allowed = Palabras permitidas
admin_filter_allowed_hint = Excepciones a las palabras bloqueadas, una por línea
admin_filter_reserved = Usuarios reservados adicionales
admin_filter_reserved_hint = Uno por línea
admin_filter_save = Guardar filtro
admin_filter_saved = Filtro guardado
admin_filter_field_handle = usuario
admin_filter_field_name = nombre
admin_filter_field_location = ubicación
admin_filter_field_bio = biografía
admin_scan_title = Analizar cuentas
admin_scan_desc = Busca cuentas cuyo usuario, nombre o biografía incumplan el filtro.
admin_scan_run = Analizar ahora
admin_scan_clean = Ninguna cuenta incumple el filtro.
admin_scan_found =
    { $count ->
        [one] { $count } cuenta requiere atención
       *[other] { $count } cuentas requieren atención
    }
admin_scan_reason = El campo { $field } contiene { $reason }
admin_ann_title = Nuevo anuncio
admin_ann_desc = Un banner en la parte superior de la pantalla de todos hasta que caduque o lo termines. Se puede descartar.
admin_ann_text = Mensaje
admin_ann_link = Enlace
admin_ann_link_hint = Opcional: https://… o /ruta
admin_ann_tone = Estilo
admin_ann_tone_info = Información
admin_ann_tone_success = Buenas noticias
admin_ann_tone_warning = Advertencia
admin_ann_duration = Duración
admin_ann_days =
    { $count ->
        [one] { $count } día
       *[other] { $count } días
    }
admin_ann_week = 1 semana
admin_ann_until_ended = Hasta que lo termines
admin_ann_publish = Publicar
admin_ann_published = Anuncio publicado
admin_ann_list = Anuncios hasta ahora
admin_ann_none = Aún no hay anuncios.
admin_ann_end = Terminar
admin_ann_ended = terminado
admin_ann_expired = caducado
admin_ann_inactive = No se muestra
admin_ann_until = hasta el { $date }
admin_inv_title = Códigos de invitación
admin_inv_desc = Permite que personas concretas se unan mientras los registros son solo con invitación.
admin_inv_note = Nota (para quién es)
admin_inv_uses = Usos
admin_inv_use_count =
    { $count ->
        [one] { $count } uso
       *[other] { $count } usos
    }
admin_inv_expiry = Caduca
admin_inv_week = En 1 semana
admin_inv_days =
    { $count ->
        [one] En { $count } día
       *[other] En { $count } días
    }
admin_inv_never = Nunca
admin_inv_create = Crear código
admin_inv_created = Código { $code } creado
admin_inv_created_copied = Código { $code } creado y copiado
admin_inv_list = Códigos
admin_inv_none = Aún no hay códigos de invitación.
admin_inv_state_active = Activo
admin_inv_state_revoked = Revocado
admin_inv_state_used = Agotado
admin_inv_state_expired = Caducado
admin_inv_used = { $uses } de { $max } usos
admin_inv_expires = caduca el { $date }
admin_inv_copy = Copiar código { $code }
admin_inv_copied = Código copiado
admin_inv_copy_failed = No se pudo copiar. El código es { $code }.
admin_inv_revoke = Revocar
admin_inv_revoke_title = ¿Revocar esta invitación?
admin_inv_revoke_body = Quienes ya se unieron con ella conservan sus cuentas.
settings_devices = Dispositivos y sesiones
settings_devices_desc = Mira dónde tienes la sesión iniciada y ciérrala a distancia
settings_passkeys = Llaves de acceso
settings_passkeys_desc = Inicia sesión con tu huella, rostro, bloqueo de pantalla o una llave de seguridad
settings_affiliation = Afiliación
settings_affiliation_desc = Organizaciones a las que perteneces o las cuentas de tu organización
settings_bots = Bots y API
settings_bots_desc = Tokens de API y la etiqueta «Automatizada»
settings_saved = Guardado
passkey_title = Tus llaves de acceso
passkey_desc = Inicia sesión con tu huella, rostro, bloqueo de pantalla o llave de seguridad en lugar de una contraseña. Las llaves de acceso no se pueden suplantar con phishing y no se guarda nada secreto en nuestro servidor.
passkey_none = Aún no hay llaves de acceso.
passkey_add = Añadir llave de acceso
passkey_added = Llave de acceso «{ $name }» añadida
passkey_default_name = Llave de acceso
passkey_added_ago = añadida { $time }
passkey_used_ago = último uso { $time }
passkey_never_used = nunca usada
passkey_rename = Cambiar nombre
passkey_rename_label = Cambiar nombre de la llave de acceso { $name }
passkey_rename_title = Cambiar nombre de la llave de acceso
passkey_name = Nombre
passkey_rename_save = Guardar
passkey_remove = Quitar
passkey_remove_label = Quitar la llave de acceso { $name }
passkey_remove_title = ¿Quitar «{ $name }»?
passkey_remove_body = Ya no podrás iniciar sesión con esta llave de acceso. Quizá también quieras eliminarla de tu dispositivo o gestor de contraseñas.
passkey_removed = Llave de acceso eliminada
passkey_unsupported = Este navegador no puede usar llaves de acceso en esta dirección.
passkey_needs_domain = Las llaves de acceso requieren que este sitio tenga un nombre de dominio, no una dirección IP.
passkey_already_here = Este dispositivo ya tiene una llave de acceso para esta cuenta.
token_scope_read = Lectura
token_scope_write = Publicar e interactuar
token_scope_messages = Mensajes directos
token_scope_read_hint = Cronologías, notificaciones, perfiles, búsqueda y el flujo en tiempo real
token_scope_write_hint = Publicar, eliminar, me gusta, reposts, guardados, seguimientos y subidas
token_scope_messages_hint = Leer y enviar mensajes directos
token_title = Tokens de API
token_desc = Los tokens permiten que scripts y bots usen tu cuenta mediante la API. Trátalos como contraseñas: quien tenga uno puede actuar como tú dentro de sus permisos.
token_none = Aún no hay tokens.
token_name = Nombre del token
token_name_hint = Por ejemplo: bot del clima
token_scopes = Permisos
token_create = Crear token
token_used_ago = último uso { $time }
token_never_used = nunca usado
token_revoke = Revocar
token_revoke_label = Revocar el token { $name }
token_revoke_title = ¿Revocar «{ $name }»?
token_revoke_body = Todo lo que lo use dejará de funcionar.
token_revoked = Token revocado
token_secret_title = Tu nuevo token
token_secret_warning = Cópialo ahora. Por tu seguridad no se volverá a mostrar.
token_secret_label = Nuevo token de API
token_copy = Copiar
token_copied = Token copiado
token_copy_failed = No se pudo copiar. Selecciona el token y cópialo a mano.
token_done = Listo
bot_title = Cuenta automatizada
bot_desc = Muestra la etiqueta «Automatizada» junto al nombre para que se sepa que publica un bot.
bot_switch = Esta es una cuenta automatizada
bot_owner = Gestionado por
bot_owner_hint = El usuario de la persona u organización detrás del bot. Opcional.
bot_owner_save = Guardar
bot_docs = Documentación para desarrolladores
bot_docs_desc = Endpoints, eventos en tiempo real y bots de ejemplo
aff_yours_title = Tu afiliación
aff_yours_desc = Una organización puede invitarte a mostrar su insignia junto a tu nombre. Nada cambia hasta que aceptes, y puedes salir cuando quieras.
aff_none = No estás afiliado a ninguna organización.
aff_invites_title = Invitaciones
aff_accept = Aceptar
aff_decline = Rechazar
aff_leave = Salir
aff_leave_title = ¿Salir de esta organización?
aff_leave_body = Su insignia desaparece de tu perfil.
aff_remove_title = ¿Quitar a { $handle }?
aff_remove_body = Tu insignia desaparece de su perfil.
aff_remove = Quitar
aff_remove_label = Quitar a { $handle }
aff_ended = Afiliación terminada
aff_declined = Invitación rechazada
aff_accepted = Afiliación añadida
aff_invited = Invitación enviada
aff_org_title = Tus afiliados
aff_org_desc = Invita a personas a mostrar tu insignia junto a su nombre. Tienen que aceptar.
aff_handle = Usuario
aff_label = Etiqueta
aff_label_hint = Opcional, como «Ingeniero»
aff_invite = Enviar invitación
aff_members_none = Aún no hay afiliados.
aff_pending = Invitado
aff_not_org = ¿Eres una organización? Pide a un administrador de esta instancia que marque tu cuenta.
auth_passkey = Iniciar sesión con una llave de acceso
auth_or = o
auth_invite = Código de invitación
auth_invite_hint = Esta instancia solo admite personas nuevas por invitación.
auth_invite_required = Introduce el código de invitación que te dieron.
landing_developers = Desarrolladores
landing_invite = Los registros son solo por invitación. ¿Tienes un código? Crea tu cuenta.
dev_title = Desarrolladores
dev_hero = Bots y API
dev_hero_sub = Crea bots, scripts e integraciones para { $name }
dev_intro_h = Crea para { $name }
dev_intro_p = Todo lo que hacen las aplicaciones de tuita pasa por una pequeña API JSON, y los bots y scripts también pueden usarla. Necesitas una cuenta en { $name } y un token de API personal. No hay registro de aplicaciones ni baile de OAuth.
dev_intro_1 = Crea una cuenta para tu bot o usa la tuya.
dev_intro_2 = En Ajustes, Bots y API, activa «Esta es una cuenta automatizada» para que la gente vea una etiqueta «Automatizada», e indica quién lo gestiona.
dev_intro_3 = Crea un token con los permisos que necesites y mantenlo en secreto.
dev_auth_h = Autenticación
dev_auth_p = Envía el token en la cabecera Authorization. Cada solicitud y respuesta es JSON (UTF-8), salvo las subidas, que son multipart.
dev_auth_prefix = Un token es tuita_ seguido de 43 letras, dígitos, guiones y guiones bajos. Solo se guarda un hash, así que un token perdido no se puede mostrar de nuevo: crea uno nuevo. Un token tiene uno o más de estos permisos:
dev_auth_refused = Un token no puede cambiar contraseñas, sesiones, tokens ni llaves de acceso, no puede exportar ni eliminar la cuenta y no puede usar la API de administración. Eso requiere un inicio de sesión real en el sitio web y responde 403.
dev_auth_errors = Los errores llegan como JSON con un mensaje de error y un estado 4xx. 401 significa que el token es incorrecto o fue revocado, 403 que falta un permiso o la ruta está rechazada, y 429 viene con una cabecera Retry-After.
dev_post_h = Tu primera publicación
dev_post_p = Sube primero imágenes y vídeo con POST /api/v1/media (multipart) y luego adjúntalos por id en mediaIds. Una respuesta define replyTo y una cita define quoteOf.
dev_endpoints_h = Endpoints
dev_endpoints_intro = Las partes de la ruta entre llaves son marcadores de posición. Las listas paginan con ?before= igual al siguiente valor de la página anterior; limit va de 1 a 40. La columna de permiso indica el permiso que necesita un token.
dev_col_method = Método
dev_col_path = Ruta
dev_col_scope = Permiso
dev_col_what = Qué hace
dev_ep_me = La cuenta a la que pertenece el token
dev_ep_patch_me = Cambia el perfil y los ajustes de esa cuenta
dev_ep_bot = Activa o desactiva la etiqueta «Automatizada» e indica quién gestiona el bot
dev_ep_following = La cronología de inicio de las personas a las que sigues
dev_ep_foryou = La cronología Para ti; pagina con ?cursor=
dev_ep_notifications = Notificaciones (?filter=mentions para respuestas, menciones y citas)
dev_ep_post = Una publicación con su hilo y respuestas
dev_ep_user = Un perfil (usuario@dominio para cuentas de otros servidores)
dev_ep_user_posts = Publicaciones de alguien (?tab=replies, media o likes; con media también ?kind=photo o video)
dev_ep_search = Busca publicaciones, personas o multimedia, con operadores como from: y has:media
dev_ep_trends = Los hashtags más usados en la última semana
dev_ep_stream = Eventos en tiempo real (Server-Sent Events)
dev_ep_create = Escribe una publicación
dev_ep_thread = Escribe de 2 a 25 publicaciones como un hilo
dev_ep_delete = Elimina una de tus publicaciones
dev_ep_like = Da me gusta a una publicación (DELETE para deshacer). También /repost y /bookmark
dev_ep_follow = Sigue a alguien (DELETE para dejar de seguir)
dev_ep_media = Sube una imagen o vídeo como multipart; la respuesta incluye el id para adjuntar
dev_ep_schedule = Programa una publicación para más tarde
dev_ep_report = Denuncia una publicación o una cuenta a los moderadores
dev_ep_conversations = Tus conversaciones de mensajes directos
dev_ep_message = Envía un mensaje directo; el cuerpo es JSON con un campo text
dev_realtime_h = Eventos en tiempo real
dev_realtime_p = GET /api/v1/stream es un flujo de Server-Sent Events. Envía un evento notification por cada me gusta, repost, seguimiento, respuesta, mención o cita, un evento message para los mensajes directos y badges con los contadores de no leídos. Envía una línea de comentario cada 25 segundos y pide a los clientes que reintenten tras 5 segundos; vuelve a conectar si se corta la conexión.
dev_example_h = Ejemplo: un bot que responde a menciones
dev_example_node = Node 22 o posterior, sin paquetes:
dev_example_python = Lo mismo en Python, usando solo requests:
dev_limits_h = Límites
dev_limit_rate = Cada token puede hacer 600 solicitudes cada 10 minutos. Si se supera el límite, el servidor responde 429 con una cabecera Retry-After.
dev_limit_tokens = Una cuenta puede tener hasta 20 tokens y crear hasta 20 tokens por hora.
dev_limit_posting = Publicar, seguir, denunciar y las demás acciones tienen sus propios límites por cuenta.
dev_rules_h = Reglas para bots
dev_rule_1 = Activa la etiqueta Automatizada e indica quién gestiona el bot.
dev_rule_2 = Responde o escribe solo a quien lo haya pedido: mencionó al bot, lo siguió o le escribió primero. No envíes menciones ni mensajes directos no solicitados.
dev_rule_3 = Mantén la calma: quédate muy por debajo de los límites anteriores.
dev_rule_4 = Cumple los Términos y las reglas de la instancia. Los moderadores pueden suspender bots como cualquier cuenta.
admin_act_log_admin_bootstrap = Primer administrador configurado
admin_act_log_announcement_create = Anuncio publicado
admin_act_log_announcement_end = Anuncio terminado
admin_act_log_instance_settings = Ajustes de la instancia cambiados
admin_act_log_invite_create = Invitación creada
admin_act_log_invite_revoke = Invitación revocada
admin_act_log_post_delete = Publicación eliminada
admin_act_log_report_dismiss = Denuncia desestimada
admin_act_log_user_delete = Cuenta eliminada
admin_act_log_user_org = Marcada como organización
admin_act_log_user_unorg = Estado de organización eliminado
admin_act_log_user_purge_posts = Todas las publicaciones eliminadas
admin_act_log_user_rename = Cuenta renombrada
admin_act_log_user_role = Rol cambiado
admin_act_log_user_suspend = Cuenta suspendida
admin_act_log_user_unsuspend = Suspensión de la cuenta quitada
admin_act_log_user_unverify = Verificación eliminada
admin_act_log_user_verify = Cuenta verificada
admin_act_log_domain_block = Instancia bloqueada
admin_act_log_domain_unblock = Instancia desbloqueada
admin_act_log_federation_retry = Reintento de entrega solicitado
admin_fed_stat_domains = Instancias conocidas
admin_fed_stat_domains_sub = vistas por esta instancia
admin_fed_stat_accounts = Cuentas remotas
admin_fed_stat_accounts_sub = conocidas de otros lugares
admin_fed_stat_posts = Publicaciones remotas
admin_fed_stat_posts_sub = almacenadas localmente
admin_fed_stat_queue = En cola
admin_fed_stat_queue_sub = entregas en espera
admin_fed_queue_title = Cola de entregas
admin_fed_queue_desc = Actividades firmadas a la espera de llegar a otras instancias. Las fallidas se reintentan con retrasos crecientes y se descartan tras 12 intentos.
admin_fed_queue_empty = No hay nada en espera. Todas las entregas se han enviado.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } entrega en espera
       *[other] { $compact } entregas en espera
    }
admin_fed_queue_due = { $count } pendientes ahora
admin_fed_queue_oldest = La más antigua en cola: { $date }
admin_fed_retry = Reintentar ahora
admin_fed_retry_done =
    { $count ->
        [one] { $compact } entrega se reintentará ahora.
       *[other] { $compact } entregas se reintentarán ahora.
    }
admin_fed_failing = Instancias con fallos
admin_fed_failing_none = Ninguna instancia está fallando.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } entrega
       *[other] { $compact } entregas
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } intento
       *[other] { $compact } intentos
    }
admin_fed_failing_next = próximo intento { $date }
admin_fed_failing_error = Último error
admin_fed_failing_no_error = No se registró ningún error
admin_fed_actor_title = Actor de la instancia
admin_fed_actor_desc = Otras instancias comprueban con esta clave las solicitudes que firma esta instancia.
admin_fed_actor_id = Actor
admin_fed_actor_key = ID de clave
admin_fed_actor_fingerprint = Huella SHA-256
admin_fed_actor_copy = Copiar huella
admin_fed_actor_copied = Huella copiada
admin_fed_actor_copy_failed = No se pudo copiar. La huella es { $fingerprint }
admin_fed_blocks_title = Instancias bloqueadas
admin_fed_blocks_desc = Silenciar oculta las cuentas de una instancia de las cronologías públicas y de la búsqueda. Suspender las oculta en todas partes, descarta lo que envían y detiene las entregas hacia ellas.
admin_fed_domain = Dominio
admin_fed_domain_hint = example.social, o un enlace a cualquier página suya
admin_fed_severity = Gravedad
admin_fed_silence = Silenciar
admin_fed_suspend = Suspender
admin_fed_reason = Motivo
admin_fed_reason_hint = Opcional, solo lo ve el equipo
admin_fed_block = Bloquear instancia
admin_fed_blocked = { $domain } está bloqueada
admin_fed_none = No hay instancias bloqueadas.
admin_fed_list = Bloqueos
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } cuenta conocida
       *[other] { $compact } cuentas conocidas
    }
admin_fed_row_no_reason = Sin motivo indicado
admin_fed_row_changed = { $domain } ahora está en { $severity }
admin_fed_remove = Quitar
admin_fed_remove_label = Quitar el bloqueo de { $domain }
admin_fed_remove_title = ¿Quitar el bloqueo de { $domain }?
admin_fed_remove_body = Sus cuentas y publicaciones vuelven y se reanudan las entregas hacia ella.
admin_fed_removed = { $domain } ya no está bloqueada
remote_domain_title = Cuenta en { $domain }
remote_view_original = Ver en { $domain }
visibility_unlisted = No listada
visibility_unlisted_hint = No se muestra en las cronologías públicas
visibility_followers = Solo seguidores
visibility_followers_hint = Solo los seguidores pueden verla
visibility_direct = Directa
visibility_direct_hint = Solo las personas mencionadas pueden verla
follow_requested = Solicitado
follow_cancel_request = Cancelar solicitud de seguimiento
follow_requested_toast = Solicitud de seguimiento enviada a { $handle }
follow_cancelled_toast = Solicitud de seguimiento cancelada
profile_locked = Aprueba seguidores
profile_locked_hint = Las solicitudes de seguimiento necesitan la aprobación de esta cuenta.
profile_original = Ver perfil en { $domain }
profile_remote_note = Una copia de { $domain }. Las cifras y publicaciones son lo que esta instancia ha visto.
remote_lookup_loading = Buscando a { $handle } en { $domain }…
remote_lookup_notfound_title = No existe esa cuenta
remote_lookup_notfound_body = { $handle } no existe, o { $domain } no la comparte.
remote_lookup_failed_title = No se pudo cargar a { $handle }
explore_lookup = Buscando { $target }…
explore_lookup_empty_body = No se encontró esa cuenta o publicación. Puede que la otra instancia esté desconectada o que no la comparta.
language_label = Idioma
oauth_title = Autorizar una aplicación
oauth_loading = Comprobando la solicitud…
oauth_wants = Esta aplicación quiere usar tu cuenta { $handle }.
oauth_name_warning = Cualquiera puede elegir cualquier nombre de aplicación. Aprueba solo aplicaciones de confianza.
oauth_website = Sitio web
oauth_allow = Podrá
oauth_scope_read = Leer tu perfil, publicaciones, seguimientos, listas, cronologías y notificaciones, y los mensajes directos que no tienen cifrado de extremo a extremo.
oauth_scope_write = Publicar, editar y eliminar por ti, dar me gusta, seguir, bloquear, silenciar y cambiar tu perfil y ajustes.
oauth_scope_messages = Leer y enviar tus mensajes directos.
oauth_push_note = Las notificaciones push no son compatibles. Una aplicación que las pida no las recibirá.
oauth_redirect = Después de responder, se te enviará a
oauth_redirect_oob = La aplicación no recibe redirección. Aquí se muestra un código para que lo pegues en ella.
oauth_approve = Autorizar
oauth_deny = Denegar
oauth_switch = Usar otra cuenta
oauth_sent_approved = Autorizado. Te devolvemos a { $host }. Si no pasa nada, cierra esta pestaña.
oauth_sent_denied = Denegado. Te devolvemos a { $host }. Si no pasa nada, cierra esta pestaña.
oauth_denied = Denegado. No se compartió nada con la aplicación.
oauth_code_title = Código de autorización
oauth_code_body = Pega este código en la aplicación. Funciona una sola vez y caduca en 10 minutos.
oauth_code_copy = Copiar código
oauth_code_copied = Código copiado
oauth_code_copy_failed = No se pudo copiar. Selecciona el código y cópialo a mano.
oauth_error_title = No se puede autorizar esta solicitud
oauth_error_note = No se envió nada a la aplicación.
oauth_error_invalid_request = La solicitud está mal formada o pide volver a una dirección que la aplicación no registró. Vuelve a la aplicación e inténtalo de nuevo.
oauth_error_invalid_client = Esta aplicación no está registrada aquí.
oauth_error_invalid_scope = La aplicación pide un acceso que tuita no ofrece, o más del que registró.
oauth_error_unsupported_response_type = La aplicación pide un tipo de inicio de sesión que tuita no admite.
oauth_error_rate = Has respondido a demasiadas solicitudes. Espera unos minutos e inténtalo de nuevo.
oauth_error_unreachable = No se pudo conectar con tuita. Comprueba tu conexión e inténtalo de nuevo.
oauth_home = Ir a tuita
share_send_chat = Enviar por chat
share_download = Descargar contenido
share_download_n = Descargar contenido { $number }
share_chat_title = Enviar publicación por chat
share_chat_sent = Enviado a { $handle }
hover_card_label = Tarjeta de perfil de { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Tipo de contenido
profile_media_all = Todo el contenido
profile_media_photos = Fotos
profile_media_videos = Vídeos
profile_empty_videos_title = Aún no hay vídeos
profile_empty_videos_body = Las publicaciones con vídeos aparecerán aquí.
account_add_existing = Añadir una cuenta existente
rail_label = Búsqueda y sugerencias
rail_search_submit = Buscar
rail_show_more = Mostrar más
rail_footer = Enlaces
chat_new = Nuevo chat
chat_search = Buscar conversaciones
chat_search_none = Ninguna conversación coincide con tu búsqueda.
chat_start_title = Iniciar conversación
chat_start_body = Elige una conversación de la lista o inicia una nueva.
explore_sections = Secciones de Explorar
explore_see_more = Mostrar más
history_title = Historial
history_tabs = Secciones del historial
history_likes_label = Publicaciones que te gustan
video_play = Reproducir
video_pause = Pausar
video_mute = Silenciar
video_unmute = Activar sonido
video_volume = Volumen
video_seek = Buscar en el vídeo
video_position = { $current } de { $total }
video_speed = Velocidad de reproducción
video_fullscreen = Pantalla completa
video_exit_fullscreen = Salir de pantalla completa
video_pip = Imagen en imagen
video_exit_pip = Salir de imagen en imagen
video_error = No se puede reproducir este vídeo.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Ocultar panel
lightbox_show_panel = Mostrar panel
lightbox_view_post = Ver publicación
lightbox_panel = Publicación y respuestas
emotepick_tabs = Conjuntos de emojis
emotepick_tab_emoji = Estándar
emotepick_tab_custom = Personalizados
admin_emote_tab = Emotes
admin_emote_title = Emotes personalizados
admin_emote_desc = Los emotes pertenecen a toda la instancia. Cualquiera puede usarlos como :shortcode: en publicaciones, nombres y mensajes. Un shortcode no se puede renombrar; elimina el emote y súbelo de nuevo.
admin_emote_shortcode = Shortcode
admin_emote_shortcode_hint = De 2 a 32 letras, números o guiones bajos. Se muestra como :shortcode:.
admin_emote_invalid = Usa solo letras, números y guiones bajos, de 2 a 32 caracteres.
admin_emote_category = Categoría
admin_emote_category_hint = Opcional. Agrupa los emotes en el selector.
admin_emote_file = Imagen del emote
admin_emote_file_hint = PNG, JPEG, WebP o GIF, hasta 256 KB. Suelta un archivo aquí o elige uno.
admin_emote_file_type = Ese archivo no es una imagen PNG, JPEG, WebP ni GIF.
admin_emote_file_size = La imagen es demasiado grande (máx. 256 KB).
admin_emote_choose_file = Elegir imagen
admin_emote_change_file = Cambiar imagen
admin_emote_upload = Subir emote
admin_emote_upload_failed = No se pudo subir el emote. Inténtalo de nuevo.
admin_emote_uploaded = Se añadió :{ $shortcode }:.
admin_emote_list = Emotes de esta instancia
admin_emote_none = Aún no hay emotes personalizados.
admin_emote_no_category = Sin categoría
admin_emote_category_for = Categoría de :{ $shortcode }:
admin_emote_saved = Categoría guardada.
admin_emote_delete = Eliminar :{ $shortcode }:
admin_emote_delete_title = ¿Eliminar este emote?
admin_emote_delete_body = :{ $shortcode }: deja de mostrarse como imagen en todas partes y aparece como texto plano. Las publicaciones que lo usaban conservan el texto.
admin_emote_delete_confirm = Eliminar emote
admin_emote_deleted = Emote eliminado.
admin_act_log_emoji_create = Emote añadido
admin_act_log_emoji_update = Emote modificado
admin_act_log_emoji_delete = Emote eliminado
e2ee_settings = Mensajes cifrados
e2ee_settings_desc = Frase de contraseña, este dispositivo y restablecimiento
e2ee_status_label = Estado
e2ee_status_none = Desactivado. Actívalo para enviar y leer mensajes con cifrado de extremo a extremo.
e2ee_status_locked = Bloqueado en este dispositivo. Introduce tu frase de contraseña para leer aquí tus mensajes.
e2ee_status_ready = Activado y desbloqueado en este dispositivo.
e2ee_keys_error = No se pudieron comprobar tus claves de cifrado.
e2ee_setup_title = Activar los mensajes cifrados
e2ee_setup_body = Los mensajes a personas de este servidor se cifran en tu navegador. El servidor solo guarda texto que no puede leer. Con una frase de contraseña puedes desbloquear tus mensajes en otro dispositivo.
e2ee_setup_warning = Tu frase de contraseña no se puede recuperar. Si la olvidas, puedes restablecer el cifrado, pero perderás tus mensajes anteriores.
e2ee_setup_submit = Activar
e2ee_setup_done = Los mensajes cifrados están activados
e2ee_unlock_title = Desbloquear mensajes cifrados
e2ee_unlock_body = Introduce tu frase de contraseña para leer y enviar mensajes cifrados en este dispositivo.
e2ee_unlock_submit = Desbloquear
e2ee_unlock_wrong = Esa frase de contraseña no es correcta.
e2ee_unlock_done = Desbloqueado
e2ee_passphrase = Frase de contraseña
e2ee_passphrase_confirm = Confirma la frase de contraseña
e2ee_passphrase_hint = Al menos 10 caracteres.
e2ee_passphrase_short = Usa al menos 10 caracteres.
e2ee_passphrase_mismatch = Las frases de contraseña no coinciden.
e2ee_failed = Algo salió mal. Inténtalo de nuevo.
e2ee_unsupported = Este navegador no admite mensajes cifrados.
e2ee_forget = Olvidar en este dispositivo
e2ee_forget_hint = Quita de este navegador tu clave y los contactos en los que confías. Tu frase de contraseña los desbloquea de nuevo.
e2ee_forget_done = Olvidado en este dispositivo
e2ee_reset = Restablecer el cifrado
e2ee_reset_hint = Crea una clave nueva. Los mensajes que enviaste y recibiste antes dejan de ser legibles.
e2ee_reset_title = ¿Restablecer el cifrado?
e2ee_reset_body = Tus mensajes anteriores dejan de ser legibles, para ti y para las personas a las que escribiste, y no se pueden recuperar. Se pedirá a tus contactos que confíen en tu nueva clave. Elige una nueva frase de contraseña.
e2ee_reset_submit = Restablecer
e2ee_reset_done = Se restableció el cifrado
e2ee_note_sealed = Cifrado de extremo a extremo
e2ee_note_remote = Sin cifrado de extremo a extremo: esta persona está en otro servidor.
e2ee_note_plain = Sin cifrado de extremo a extremo.
e2ee_plain_mark = Sin cifrar
e2ee_preview = Mensaje cifrado
e2ee_line_sealed = Mensaje cifrado
e2ee_line_own_key = Enviado a una clave que ya no tienes
e2ee_line_peer_key = Enviado con una clave en la que aún no confías
e2ee_line_unreadable = No se pudo abrir
e2ee_line_unsupported = Este navegador no puede abrir mensajes cifrados
e2ee_block_peer_off = { $handle } aún no ha activado los mensajes cifrados.
e2ee_block_self_off = Primero activa los mensajes cifrados en Ajustes.
e2ee_block_unlock = Primero desbloquea los mensajes cifrados en Ajustes.
e2ee_block_downgrade = { $handle } ya usó mensajes cifrados antes. Esta conversación no volverá a mensajes sin cifrar.
e2ee_block_changed = Confía en la nueva clave de { $handle } para escribirle.
e2ee_block_error = No se pudieron comprobar tus claves de cifrado.
e2ee_changed_title = La clave de cifrado de { $name } cambió
e2ee_changed_body = Puede que haya restablecido el cifrado o se haya pasado a un dispositivo nuevo, o que alguien se interponga. Compara con esa persona el nuevo número de seguridad antes de confiar en él. No podrás enviar hasta que lo hagas.
e2ee_changed_old = Número de seguridad en el que confiabas
e2ee_changed_new = Número de seguridad nuevo
e2ee_trust = Confiar en la clave nueva
e2ee_safety_open = Número de seguridad
e2ee_safety_title = Número de seguridad
e2ee_safety_body = Compara este número con { $name } en persona o en una llamada, no en este chat. Si coincide en ambas pantallas, nadie se interpone entre los dos.

## 0031
composer_upload_failed = Falló la subida. Inténtalo de nuevo.
composer_send_failed = No se pudo conectar con tuita. Comprueba tu conexión e inténtalo de nuevo.
composer_open_failed = No se pudo abrir el editor de publicaciones. Inténtalo de nuevo.
