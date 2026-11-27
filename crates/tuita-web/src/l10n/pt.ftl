app_name = tuita
loading = Carregando
close = Fechar
dismiss = Dispensar
notifications = Notificações
avatar_more = mais { $count }
char_counter_left =
    { $count ->
        [one] { $count } caractere restante
       *[other] { $count } caracteres restantes
    }
char_counter_over =
    { $count ->
        [one] { $count } caractere acima do limite
       *[other] { $count } caracteres acima do limite
    }
not_found_title = Nada aqui
not_found_body = Esta página não existe ou foi movida.
not_found_back = Voltar
tagline = Cada publicação, duas tintas.
error_title = Algo deu errado
cancel = Cancelar
retry = Tentar de novo
back = Voltar
view = Ver
follow = Seguir
following = Seguindo
follows_you = Segue você
verified = Conta verificada
bot_account = Conta automatizada
bot_label = Bot
link_copied = Link copiado para a área de transferência
text_copied = Texto copiado
time_now = agora
time_minutes = { $n } min
time_hours = { $n } h
time_days = { $n } d
nav_main = Principal
nav_home = Início
nav_explore = Explorar
nav_notifications = Notificações
nav_bookmarks = Itens salvos
nav_profile = Perfil
nav_settings = Configurações
nav_compose = Publicar
nav_menu = Menu
badge_unread = Não lidas: { $count }
account_menu = Contas
account_switch = Trocar de conta
account_switch_to = Trocar para { $handle }
account_add = Adicionar uma conta
account_logout = Sair de { $handle }
account_logout_all = Sair de todas as contas
home_feed = Linha do tempo
home_for_you = Para você
home_following = Seguindo
home_empty_title = Nada aqui ainda
home_empty_body = Seja o primeiro a publicar algo.
home_following_empty_title = Seu círculo está quieto
home_following_empty_body = Siga pessoas para encher esta linha do tempo com as publicações e repostagens delas.
home_find_people = Encontrar pessoas
new_posts =
    { $count ->
        [one] Mostrar { $count } nova publicação
       *[other] Mostrar { $count } novas publicações
    }
new_posts_many = Mostrar mais de 99 novas publicações
composer_title = Nova publicação
composer_reply_title = Responder
composer_quote_title = Citar publicação
composer_placeholder = O que está acontecendo?
composer_reply_placeholder = Publique sua resposta
composer_text_label = Texto da publicação
composer_replying_to = Respondendo a { $handle }
composer_cw_label = Aviso de conteúdo
composer_cw_placeholder = Aviso de conteúdo, por exemplo: spoilers
composer_cw_toggle = Adicionar um aviso de conteúdo
composer_sensitive_toggle = Marcar mídia como sensível
composer_options = Opções da publicação
composer_policy_label = Quem pode responder
composer_suggestions = Sugestões
composer_tag_posts =
    { $count ->
        [one] { $compact } publicação
       *[other] { $compact } publicações
    }
composer_post = Publicar
composer_reply = Responder
policy_everyone = Todos podem responder
policy_following = Pessoas que você segue podem responder
policy_mentioned = Só pessoas que você mencionar podem responder
post_sent = Publicação enviada
post_by = Publicação de { $name }
post_reposted_by = { $name } repostou
post_pinned = Publicação fixada
post_replying_to = Respondendo a { $handle }
post_replying_unavailable = Respondendo a uma publicação indisponível
post_unavailable = Esta publicação não está disponível.
post_menu = Mais
post_copy_link = Copiar link
post_copy_text = Copiar texto
post_pin = Fixar no perfil
post_unpin = Desafixar do perfil
post_pinned_toast = Fixado no seu perfil
post_unpinned = Desafixado do seu perfil
post_delete = Excluir publicação
post_delete_title = Excluir publicação?
post_delete_body = Isso não pode ser desfeito. Ela será removida do seu perfil, das linhas do tempo de quem segue você e da pesquisa.
post_deleted = Publicação excluída
post_view_profile = Ver { $handle }
post_mute = Silenciar { $handle }
post_block = Bloquear { $handle }
post_reply_label = Responder ({ $count })
post_reply_following_only = Só pessoas que o autor segue podem responder.
post_reply_mentioned_only = Só pessoas mencionadas nesta publicação podem responder.
post_repost_menu = Opções de repostagem
post_repost = Repostar
post_undo_repost = Desfazer repostagem
post_quote = Citar
post_reposts_label = Repostagens e citações ({ $count })
post_like_label = Curtir ({ $count })
post_bookmark_label = Salvar
post_share = Compartilhar ou copiar link
cw_show = Mostrar
cw_hide = Ocultar
cw_label = Aviso de conteúdo: { $reason }
muted_toast = { $handle } silenciado
unmuted_toast = { $handle } não está mais silenciado
blocked_toast = { $handle } bloqueado
unblocked_toast = { $handle } desbloqueado
block_title = Bloquear { $handle }?
block_body = A pessoa não poderá seguir você e você não verá as publicações dela. Ela não será avisada.
block_confirm = Bloquear
notify_like = @{ $handle } curtiu sua publicação
notify_repost = @{ $handle } repostou sua publicação
notify_reply = @{ $handle } respondeu a você
notify_mention = @{ $handle } mencionou você
notify_quote = @{ $handle } citou sua publicação
notify_follow = @{ $handle } começou a seguir você
notify_follow_request = @{ $handle } pediu para seguir você
notify_poll = Uma enquete em que você votou terminou
notify_affiliation = @{ $handle } convidou você para se afiliar
shortcuts_title = Atalhos de teclado
shortcut_group_navigation = Navegação
shortcut_group_actions = Ações
shortcut_or = /
shortcut_then = depois
shortcut_plus = +
shortcut_next_prev = Publicação seguinte / anterior
shortcut_open = Abrir publicação
shortcut_search = Pesquisar
shortcut_go_home = Início
shortcut_go_explore = Explorar
shortcut_go_notifications = Notificações
shortcut_go_bookmarks = Itens salvos
shortcut_go_profile = Seu perfil
shortcut_go_settings = Configurações
shortcut_new_post = Nova publicação
shortcut_like = Curtir
shortcut_reply = Responder
shortcut_repost = Repostar ou citar
shortcut_bookmark = Salvar
shortcut_send = Enviar publicação
shortcut_help = Mostrar esta ajuda
auth_login_title = Entrar
auth_add_title = Adicionar outra conta
auth_add_sub = Entre para alternar entre até { $max } contas neste dispositivo sem precisar sair.
auth_signup_title = Criar conta
auth_handle = Nome de usuário
auth_password = Senha
auth_display_name = Nome de exibição
auth_handle_hint = 3–15 letras, números ou sublinhados
auth_password_hint = Pelo menos 8 caracteres
auth_consent_pre = Tenho pelo menos { $minAge } anos e concordo com os{" "}
auth_consent_terms = Termos de Serviço
auth_consent_and = {" "}e com a{" "}
auth_consent_privacy = Política de Privacidade
auth_consent_end = {"."}
auth_consent_required = Aceite os Termos de Serviço e a Política de Privacidade.
auth_signin = Entrar
auth_signin_add = Entrar e trocar
auth_create = Criar conta
auth_to_signup = Novo por aqui? Crie uma conta
auth_to_login = Já tem uma conta? Entre
auth_closed = Os cadastros estão fechados nesta instância no momento.
landing_nav = Rodapé
landing_rules = Regras da casa
landing_stats = Agora mesmo
landing_people = Pessoas
landing_posts = Publicações
landing_clock = Sua hora local
landing_trends = Em alta
landing_trend =
    { $count ->
        [one] { $count } publicação
       *[other] { $count } publicações
    }
landing_join = Participe do { $name }
landing_join_sub = Crie uma conta e publique em duas tintas.
landing_closed = Os cadastros estão fechados no momento. Já tem uma conta? Entre.
landing_terms = Termos
landing_privacy = Privacidade
landing_source = Código-fonte
landing_powered = Com tecnologia tuita { $version }
docs_terms_title = Termos de Serviço
docs_privacy_title = Política de Privacidade
docs_updated = Última atualização: { $date }
docs_date = 5 de novembro de 2026
docs_see_privacy = Veja também a Política de Privacidade.
docs_see_terms = Veja também os Termos de Serviço.
docs_contact_fallback = os administradores da instância (por mensagem direta ou denúncia)
docs_terms_agreement_h = 1. Acordo
docs_terms_agreement_p = Estes Termos de Serviço (“Termos”) regem o uso do { $name }, a instância tuita em { $domain } (“o serviço”, “nós”). Ao criar uma conta ou usar o serviço, você concorda com estes Termos e com nossa Política de Privacidade. Se não concordar, não use o serviço.
docs_terms_eligibility_h = 2. Quem pode usar
docs_terms_eligibility_p = Você deve ter pelo menos { $minAge } anos ou a idade mínima de consentimento digital do lugar onde mora, se for maior. Você deve poder firmar um acordo vinculante e não estar impedido de usar o serviço pela lei aplicável.
docs_terms_account_h = 3. Sua conta
docs_terms_account_1 = Mantenha sua senha em segredo. Você é responsável pela atividade da sua conta.
docs_terms_account_2 = Seu nome de usuário não pode se passar por outra pessoa ou organização nem enganar os outros sobre quem você é.
docs_terms_account_3 = Avise-nos logo se achar que sua conta foi comprometida. Você pode encerrar as outras sessões em Configurações → Conta.
docs_terms_content_h = 4. Seu conteúdo
docs_terms_content_p1 = Você continua dono do que publica. Você nos concede uma licença mundial, não exclusiva e gratuita para armazenar, exibir, reproduzir e distribuir seu conteúdo apenas para operar e melhorar o serviço. Isso inclui enviá-lo a instâncias tuita federadas (veja a seção 6). Essa licença termina quando você exclui o conteúdo ou a conta, exceto por cópias já entregues a outras instâncias ou salvas por outras pessoas.
docs_terms_content_p2 = Você é responsável pelo que publica. Publique apenas conteúdo que você tem o direito de compartilhar.
docs_terms_rules_h = 5. Regras da comunidade
docs_terms_rules_lead = Ao usar o { $name }, você concorda em não violar a lei nem estas regras:
docs_terms_rules_p = Você também concorda em não atacar o serviço, por exemplo tentando contornar medidas de segurança, coletando dados em grande escala, enviando malware ou sobrecarregando-o.
docs_terms_federation_h = 6. Federação
docs_terms_federation_p = O { $name } pode se conectar (“federar”) com outras instâncias tuita independentes aprovadas pelos administradores. Quando você interage com pessoas de outra instância, seu perfil público, suas publicações públicas e a interação (por exemplo, seguir, curtir, repostar, responder ou enviar uma mensagem direta) são enviados a essa instância. As outras instâncias são administradas por outras pessoas, sob termos próprios. Não podemos controlar nem excluir o conteúdo depois que outra instância o recebe, mas enviamos pedidos de exclusão quando você apaga algo.
docs_terms_moderation_h = 7. Moderação e aplicação
docs_terms_moderation_p = Os moderadores podem remover conteúdo, limitar recursos ou suspender contas que violem estes Termos ou a lei. Sempre que possível, explicaremos o motivo. Qualquer pessoa pode denunciar uma publicação ou conta. As denúncias vão para os moderadores desta instância. Podemos deixar de federar com qualquer instância a qualquer momento.
docs_terms_termination_h = 8. Saída
docs_terms_termination_p = Você pode exportar seus dados ou excluir sua conta a qualquer momento em Configurações → Conta. Podemos suspender ou encerrar seu acesso se você violar estes Termos de forma grave ou repetida. Também podemos encerrar o serviço e tentaremos avisar com antecedência razoável.
docs_terms_disclaimers_h = 9. Isenções e responsabilidade
docs_terms_disclaimers_p = O serviço é fornecido “no estado em que se encontra” e “conforme a disponibilidade”, sem garantias de qualquer tipo, na máxima extensão permitida pela lei. Não somos responsáveis por danos indiretos, incidentais ou consequentes, nem por conteúdo publicado por usuários ou outras instâncias. Nada nestes Termos limita responsabilidades que a lei não permite limitar, nem seus direitos legais como consumidor.
docs_terms_changes_h = 10. Alterações
docs_terms_changes_p = Podemos atualizar estes Termos. Se a mudança for significativa, avisaremos no app antes de ela entrar em vigor. Continuar usando o serviço depois disso significa que você aceita os novos Termos.
docs_terms_contact_h = 11. Contato
docs_terms_contact_lead = Dúvidas sobre estes Termos:
docs_privacy_summary_h = Resumindo
docs_privacy_summary_1 = Sem anúncios, sem scripts de rastreamento, sem venda de dados.
docs_privacy_summary_2 = Um cookie, para manter você conectado. As preferências de tema ficam no seu navegador.
docs_privacy_summary_3 = Publicações públicas são públicas e podem chegar a instâncias tuita federadas.
docs_privacy_summary_4 = As mensagens diretas entre contas deste servidor têm criptografia de ponta a ponta quando as duas pessoas a ativam: são armazenadas como texto que o servidor não consegue ler, então um vazamento do banco de dados ou uma olhada nos dados armazenados não revela nada. As mensagens para pessoas de outros servidores, e as enviadas antes de a criptografia ser ativada, não são criptografadas dessa forma e os administradores tecnicamente podem lê-las. Este servidor também entrega o aplicativo que faz a criptografia, então um administrador que alterasse esse aplicativo poderia capturar o que você digita.
docs_privacy_summary_5 = Você pode exportar ou excluir seus dados quando quiser.
docs_privacy_controller_h = 1. Quem somos
docs_privacy_controller_p = Esta política abrange o { $name } em { $domain }, administrado pelos administradores da instância. Eles decidem como seus dados são tratados.
docs_privacy_collect_h = 2. O que coletamos
docs_privacy_collect_1 = Dados da conta: nome de usuário, nome de exibição, senha (guardada apenas como hash argon2id com salt) e os detalhes do perfil: bio, localização, site, avatar, banner, formato e cor.
docs_privacy_collect_2 = Conteúdo: publicações, imagens, respostas, curtidas, repostagens, itens salvos, seguidores, bloqueios, silenciamentos e denúncias.
docs_privacy_collect_3 = Mensagens diretas: armazenadas neste servidor para serem entregues a você e à outra pessoa. Uma mensagem com criptografia de ponta a ponta é armazenada como texto cifrado que não conseguimos ler; quem a enviou, para quem e quando continuam visíveis para nós. As demais mensagens são armazenadas como texto simples. Sua chave de criptografia é armazenada aqui apenas de uma forma bloqueada pela sua frase secreta, que nunca vemos.
docs_privacy_collect_4 = Dados de sessão: para cada dispositivo conectado, o user-agent do navegador, o endereço IP e o horário do último uso, para você poder revisar e encerrar sessões.
docs_privacy_collect_5 = Dados de segurança de curta duração: endereços IP mantidos na memória por alguns minutos para limitar o volume de requisições.
docs_privacy_collect_p = Não coletamos seu e-mail, telefone, contatos ou localização, a menos que você os digite no perfil.
docs_privacy_use_h = 3. Como usamos
docs_privacy_use_p = Somente para operar o serviço: mostrar seu perfil e publicações, montar linhas do tempo e notificações, entregar mensagens, manter as contas seguras, evitar abusos e aplicar os Termos. A linha do tempo “Para você” ordena as publicações por curtidas, repostagens, respostas e idade. Ela não cria um perfil sobre você.
docs_privacy_visibility_h = 4. O que outras pessoas veem
docs_privacy_visibility_p = Seu perfil, publicações, respostas, repostagens, curtidas, seguidores e as contas que você segue são públicos para qualquer pessoa conectada a esta instância e para instâncias federadas. Quando alguém compartilha o link de uma publicação ou perfil, apps como Discord, Slack ou iMessage podem buscar uma prévia com a publicação pública, as imagens e os detalhes públicos do seu perfil. Publicações com aviso de conteúdo ou marcadas como sensíveis têm prévia sem texto e sem imagens. Itens salvos, bloqueios, silenciamentos, denúncias e configurações são privados. Se você desativar “Mostrar-me em sugestões”, as pessoas ainda poderão encontrar você pelo nome de usuário exato.
docs_privacy_federation_h = 5. Federação
docs_privacy_federation_p = Quando você segue, responde, menciona, curte, reposta ou envia mensagem a alguém de outra instância tuita, enviamos a essa instância os dados necessários: seu perfil público, a publicação ou interação e o texto da mensagem, no caso de mensagens diretas. As mensagens diretas para outras instâncias não têm criptografia de ponta a ponta. As instâncias dos seus seguidores recebem suas publicações públicas e atualizações de perfil. Essas instâncias são independentes e guardam dados conforme suas próprias políticas. Quando você exclui uma publicação ou a conta, pedimos às instâncias federadas que apaguem as cópias, mas não podemos garantir que o façam.
docs_privacy_cookies_h = 6. Cookies e armazenamento local
docs_privacy_cookies_p = Usamos um único cookie de sessão HttpOnly, estritamente necessário. Suas preferências de tema, cor, tamanho do texto e movimento ficam no armazenamento local do navegador e nunca são enviadas a nós.
docs_privacy_retention_h = 7. Por quanto tempo guardamos os dados
docs_privacy_retention_1 = O conteúdo e os dados da conta são mantidos até você excluí-los.
docs_privacy_retention_2 = Publicações excluídas saem da vista pública na hora, e o texto e as imagens são apagados.
docs_privacy_retention_3 = Ao excluir sua conta, removemos suas publicações, curtidas, seguidores, mensagens e sessões e anonimizamos o registro da conta.
docs_privacy_retention_4 = As sessões expiram após 30 dias ou imediatamente quando você sai.
docs_privacy_retention_5 = As denúncias de moderação e o registro de auditoria da administração são mantidos pelo tempo necessário para manter o serviço seguro.
docs_privacy_rights_h = 8. Seus direitos
docs_privacy_rights_p = Dependendo de onde você mora (por exemplo, conforme a LGPD, o GDPR ou a CCPA), você pode ter o direito de acessar, corrigir, exportar ou excluir seus dados e de se opor ao tratamento ou restringi-lo. Você mesmo pode fazer a maior parte disso em Configurações → Conta: editar o perfil, baixar seus dados e excluir a conta. Para qualquer outra coisa, entre em contato:
docs_privacy_rights_end = Você também pode reclamar à autoridade local de proteção de dados.
docs_privacy_security_h = 9. Segurança
docs_privacy_security_p = As senhas são protegidas com hash scrypt. Os tokens de sessão são guardados apenas como hashes. As requisições entre instâncias federadas são assinadas com chaves Ed25519. Nenhum sistema é perfeitamente seguro, então use uma senha exclusiva.
docs_privacy_children_h = 10. Crianças
docs_privacy_children_p = O serviço não se destina a crianças abaixo da idade mínima definida nos Termos. Se achar que uma criança tem uma conta, denuncie.
docs_privacy_changes_h = 11. Alterações
docs_privacy_changes_p = Publicaremos as alterações desta política aqui e atualizaremos a data acima. Mudanças significativas serão anunciadas no app.
docs_privacy_contact_h = 12. Contato
docs_privacy_contact_lead = Dúvidas ou solicitações sobre privacidade:
profile_posts =
    { $count ->
        [one] { $compact } publicação
       *[other] { $compact } publicações
    }
profile_edit = Editar perfil
profile_more = Mais
profile_copy_link = Copiar link do perfil
profile_mute = Silenciar { $handle }
profile_unmute = Deixar de silenciar { $handle }
profile_block = Bloquear { $handle }
profile_unblock = Desbloquear { $handle }
profile_unblock_button = Desbloquear
profile_suspended = Esta conta está suspensa. Só a equipe pode vê-la.
profile_muted = Silenciado
profile_org = Organização
profile_role_admin = Administração
profile_role_moderator = Moderação
profile_joined = Entrou em { $date }
profile_following_label = Seguindo
profile_followers =
    { $count ->
        [one] Seguidor
       *[other] Seguidores
    }
profile_counts = Contagens de seguidores
profile_followed_by =
    { $count ->
        [1]
            { $others ->
               *[other] Seguido por { $a }
            }
        [2]
            { $others ->
               *[other] Seguido por { $a } e { $b }
            }
       *[other]
            { $others ->
                [one] Seguido por { $a }, { $b } e mais { $others } pessoa que você segue
               *[other] Seguido por { $a }, { $b } e mais { $others } pessoas que você segue
            }
    }
profile_banner = Banner do perfil
profile_tabs = Seções do perfil
profile_tab_posts = Publicações
profile_tab_replies = Respostas
profile_tab_media = Mídia
profile_tab_likes = Curtidas
profile_posts_label = Publicações
profile_empty_posts_self_title = Você ainda não publicou
profile_empty_posts_self_body = Suas publicações e repostagens aparecerão aqui.
profile_empty_posts_title = { $handle } não publicou nada
profile_empty_posts_body = Quando isso acontecer, as publicações aparecerão aqui.
profile_empty_replies_title = Ainda não há respostas
profile_empty_replies_body = As respostas aparecerão aqui.
profile_empty_media_title = Ainda não há fotos
profile_empty_media_body = Publicações com imagens aparecerão aqui.
profile_empty_likes_title = Ainda não há curtidas
profile_empty_likes_body = Publicações curtidas aparecerão aqui.
profile_blocked_title = Você bloqueou { $handle }
profile_blocked_body = Você não verá as publicações, e a pessoa não pode seguir você nem mandar mensagens.
profile_hidden_title = Você não pode ver estas publicações
profile_hidden_body = { $handle } limitou quem pode ver o perfil.
follows_tabs = Listas de seguidores
follows_tab_followers = Seguidores
follows_tab_following = Seguindo
follows_label_followers = Seguidores
follows_label_following = Seguindo
follows_empty_followers_title = Ainda sem seguidores
follows_empty_followers_body = Quando as pessoas seguirem esta conta, elas aparecerão aqui.
follows_empty_following_title = Não segue ninguém
follows_empty_following_body = As contas que ela segue aparecerão aqui.
thread_title = Publicação
thread_doc_title = { $name }: “{ $text }”
thread_views =
    { $count ->
        [one] Visualização
       *[other] Visualizações
    }
thread_reposts =
    { $count ->
        [one] Repostagem
       *[other] Repostagens
    }
thread_quotes =
    { $count ->
        [one] Citação
       *[other] Citações
    }
thread_likes =
    { $count ->
        [one] Curtida
       *[other] Curtidas
    }
thread_stats = Estatísticas da publicação
thread_replies = Respostas
thread_hidden = Respostas ocultas ({ $count })
thread_hidden_note = O autor ocultou estas respostas.
thread_list_likes = Curtido por
thread_list_reposts = Repostado por
thread_list_quotes = Citações
thread_list_empty = Ninguém ainda
tag_empty_title = Nenhuma publicação com { $tag }
tag_empty_body = Seja o primeiro a usar.
tag_posts_label = Publicações com { $tag }
bookmarks_title = Itens salvos
bookmarks_label = Publicações salvas
bookmarks_empty_title = Salve publicações para depois
bookmarks_empty_body = Toque no ícone de salvar em qualquer publicação para guardá-la aqui. Só você vê seus itens salvos.
settings_account = Sua conta
settings_account_desc = Perfil e senha
settings_profile = Editar perfil
settings_profile_desc = Nome, bio, formato do avatar e tinta
settings_privacy = Privacidade e segurança
settings_privacy_desc = Mensagens, descoberta, palavras silenciadas, contas bloqueadas e silenciadas
settings_notifications = Notificações
settings_notifications_desc = Escolha sobre o que você quer ser notificado
settings_appearance = Aparência
settings_appearance_desc = Tema, tinta, tamanho do texto, contraste e movimento
settings_language = Idioma
settings_language_desc = Idioma da interface
settings_language_note = Salvo neste navegador e, quando você está conectado, na sua conta.
settings_about = Sobre esta instância
settings_about_desc = Regras, termos, privacidade, versão e atalhos de teclado
settings_logout = Sair
settings_account_profile = Perfil
settings_account_joined = Entrou em { $date }
settings_account_edit = Editar perfil
settings_password_title = Alterar senha
settings_password_current = Senha atual
settings_password_new = Nova senha
settings_password_hint = Pelo menos 8 caracteres. Os outros dispositivos serão desconectados.
settings_password_confirm = Confirme a nova senha
settings_password_submit = Atualizar senha
settings_password_mismatch = As novas senhas não coincidem.
settings_password_updated = Senha atualizada. As outras sessões foram encerradas.
settings_sessions_title = Dispositivos e sessões
settings_sessions_desc = Encerre tudo o que você não reconhecer e depois altere sua senha.
settings_device_on = { $browser } no { $os }
settings_device_unknown = Dispositivo desconhecido
settings_session_current = Este dispositivo
settings_session_last_active = Última atividade: { $time }
settings_session_sign_out = Sair
settings_session_sign_out_device = Encerrar sessão em { $device }
settings_session_signed_out = Sessão encerrada
settings_sessions_revoke_all = Encerrar sessão em todos os outros dispositivos
settings_sessions_revoke_title = Encerrar a sessão nos outros dispositivos?
settings_sessions_revoke_body = Todos os dispositivos, exceto este, serão desconectados.
settings_sessions_revoke_confirm = Encerrar as outras
settings_sessions_revoked =
    { $count ->
        [one] { $count } sessão encerrada
       *[other] { $count } sessões encerradas
    }
settings_profile_preview = Prévia
settings_profile_preview_alt = Prévia do avatar
settings_profile_uploads = O envio de avatar e banner chegará em breve. Até lá, usamos o avatar e o banner gerados.
settings_profile_details = Detalhes
settings_profile_name = Nome de exibição
settings_profile_bio = Bio
settings_profile_location = Localização
settings_profile_website = Site
settings_profile_shape = Formato do avatar
settings_profile_ink = Tinta do perfil
settings_profile_save = Salvar perfil
settings_profile_saved = Perfil atualizado
settings_shape_square = Quadrado
settings_shape_circle = Círculo
settings_shape_squircle = Quadrado arredondado
settings_shape_octagon = Octógono
settings_ink_bone = Osso
settings_ink_phosphor = Fósforo
settings_ink_amber = Âmbar
settings_ink_cyan = Ciano
settings_ink_rose = Rosa
settings_dm_title = Mensagens diretas
settings_dm_desc = Quem pode começar uma conversa com você
settings_dm_label = Quem pode mandar mensagens
settings_dm_everyone = Todos
settings_dm_following = Pessoas que eu sigo
settings_dm_none = Ninguém
settings_dm_saved = Configurações de mensagens salvas
settings_discover_title = Descoberta
settings_discoverable = Mostrar-me em sugestões e pesquisas
settings_discoverable_hint = Ainda é possível encontrar você pelo nome de usuário exato
settings_show_client = Mostrar de qual dispositivo eu publico
settings_show_client_hint = Mostra o dispositivo abaixo das suas novas publicações
settings_words_title = Palavras silenciadas
settings_words_desc = Oculte das suas linhas do tempo, pesquisas e respostas as publicações com estas palavras, frases ou #hashtags. Suas próprias publicações nunca são ocultadas.
settings_words_label = Palavra para silenciar
settings_words_placeholder = Adicione uma palavra, frase ou #hashtag
settings_words_add = Silenciar
settings_words_remove = Deixar de silenciar { $word }
settings_words_none = Nenhuma palavra silenciada.
settings_words_limit = Você pode silenciar até { $count } palavras.
settings_blocked_title = Contas bloqueadas
settings_blocked_desc = Elas não podem seguir você, mandar mensagens, nem ver ou interagir com suas publicações. Elas não são avisadas.
settings_blocked_empty = Você não bloqueou ninguém.
settings_muted_title = Contas silenciadas
settings_muted_desc = Você não verá as publicações nem notificações. A pessoa ainda pode seguir você e mandar mensagens.
settings_muted_empty = Você não silenciou ninguém.
settings_unblock = Desbloquear
settings_unmute = Deixar de silenciar
settings_unblock_user = Desbloquear { $handle }
settings_unmute_user = Deixar de silenciar { $handle }
settings_notif_title = Notificar-me sobre
settings_notif_desc = Contas silenciadas e bloqueadas nunca notificam você.
settings_notif_like = Curtidas
settings_notif_like_hint = Quando alguém curte sua publicação
settings_notif_repost = Repostagens
settings_notif_repost_hint = Quando alguém reposta sua publicação
settings_notif_reply = Respostas
settings_notif_reply_hint = Quando alguém responde a você
settings_notif_mention = Menções
settings_notif_mention_hint = Quando alguém menciona você
settings_notif_quote = Citações
settings_notif_quote_hint = Quando alguém cita sua publicação
settings_notif_follow = Novos seguidores
settings_notif_follow_hint = Quando alguém segue você
settings_desktop_title = Notificações na área de trabalho
settings_desktop_desc = Aparece como notificação do sistema apenas enquanto esta aba está em segundo plano.
settings_desktop_state = Permissão: { $state }
settings_desktop_allowed = Permitido
settings_desktop_blocked = Bloqueado
settings_desktop_not_asked = Não solicitado
settings_desktop_enable = Ativar
settings_desktop_blocked_toast = As notificações deste site estão bloqueadas nas configurações do navegador.
settings_desktop_unsupported = Este navegador não oferece suporte a notificações na área de trabalho.
settings_theme_title = Tema
settings_appearance_note = Salvo neste dispositivo e aplicado na hora.
settings_theme_mode = Tema
settings_theme_system = Sistema
settings_theme_light = Claro
settings_theme_dark = Escuro
settings_theme_ink = Tinta
settings_theme_contrast = Contraste
settings_contrast_normal = Normal
settings_contrast_more = Mais
settings_reading_title = Texto e movimento
settings_theme_text_size = Tamanho do texto
settings_theme_motion = Movimento
settings_motion_system = Sistema
settings_motion_reduce = Reduzir
settings_preview_title = Prévia
settings_preview_text = É assim que as publicações vão aparecer com suas configurações.
settings_about_domain = Domínio
settings_about_version = Versão
settings_about_contact = Contato
settings_about_terms = Termos de Serviço
settings_about_privacy = Política de Privacidade
settings_about_source = Código-fonte
settings_about_rules = Regras
explore_search_label = Pesquise publicações, pessoas e tags
explore_search_placeholder = Pesquise publicações, pessoas, #tags
explore_clear = Limpar pesquisa
explore_tips = Dicas de pesquisa
explore_tips_title = Operadores de pesquisa
explore_op_from = Publicações de uma conta
explore_op_to = Respostas ou menções a uma conta
explore_op_media = Publicações com imagens (ou has:poll, has:links)
explore_op_since = Publicado a partir de uma data
explore_op_until = Publicado até uma data
explore_op_phrase = Palavras nesta ordem
explore_op_exclude = Excluir publicações com uma palavra
explore_results = Resultados da pesquisa
explore_tab_posts = Publicações
explore_tab_people = Pessoas
explore_tab_media = Mídia
explore_people = Pessoas
explore_posts = Publicações
explore_recent = Pesquisas recentes
explore_recent_clear = Limpar
explore_trending = Em alta
explore_trend_posts =
    { $count ->
        [one] { $compact } publicação
       *[other] { $compact } publicações
    }
explore_trends_empty = Ainda não há tendências. Use uma #hashtag!
explore_suggestions = Quem seguir
explore_suggestions_empty = Você segue todo mundo. Impressionante.
explore_no_posts_title = Nenhuma publicação para “{ $q }”
explore_no_posts_body = Tente outra palavra ou hashtag.
explore_no_people_title = Nenhuma pessoa para “{ $q }”
explore_no_people_body = Tente um nome, um nome de usuário ou @usuario@instancia para alguém de outra instância.
explore_no_media_title = Nenhuma imagem para “{ $q }”
explore_no_media_body = Publicações com imagens correspondentes aparecerão aqui.
notif_filter = Filtro de notificações
notif_tab_all = Todas
notif_tab_mentions = Menções
notif_empty_all_title = Ainda não há notificações
notif_empty_all_body = Curtidas, respostas, repostagens e novos seguidores aparecerão aqui.
notif_empty_mentions_title = Ainda não há menções
notif_empty_mentions_body = Respostas, menções e citações das suas publicações aparecem aqui.
notif_who_two = { $first } e { $second }
notif_who_many =
    { $count ->
        [one] { $first }, { $second } e mais { $count } pessoa
       *[other] { $first }, { $second } e mais { $count } pessoas
    }
notif_verb_like =
    { $count ->
        [one] curtiu sua publicação
       *[other] curtiram sua publicação
    }
notif_verb_repost =
    { $count ->
        [one] repostou sua publicação
       *[other] repostaram sua publicação
    }
notif_verb_follow =
    { $count ->
        [one] começou a seguir você
       *[other] começaram a seguir você
    }
notif_verb_follow_request =
    { $count ->
        [one] pediu para seguir você
       *[other] pediram para seguir você
    }
notif_verb_reply =
    { $count ->
        [one] respondeu a você
       *[other] responderam a você
    }
notif_verb_mention =
    { $count ->
        [one] mencionou você
       *[other] mencionaram você
    }
notif_verb_quote =
    { $count ->
        [one] citou sua publicação
       *[other] citaram sua publicação
    }
notif_verb_poll =
    { $count ->
        [one] encerrou uma enquete em que você votou
       *[other] encerraram uma enquete em que você votou
    }
notif_verb_affiliation =
    { $count ->
        [one] convidou você para se afiliar
       *[other] convidaram você para se afiliar
    }
notif_cw_snippet = ⚠ { $cw }
composer_schedule = Agendar
composer_post_all = Publicar tudo
composer_err_type = Esse arquivo não é uma imagem ou vídeo que o tuita possa usar (PNG, JPEG, GIF, WebP, MP4, WebM ou MOV).
composer_err_image_size = A imagem é grande demais (máx. 10 MB).
composer_err_video_size = Os vídeos podem ter até { $mb } MB.
composer_err_max_images = Você pode anexar até 4 imagens.
composer_err_exclusive = Uma publicação pode ter um vídeo, imagens ou uma enquete, mas só uma dessas coisas.
composer_extra_placeholder = Adicionar outra publicação
composer_extra_label = Publicação { $n } da sequência
composer_extra_remove = Remover publicação { $n } da sequência
composer_schedule_chip = Será publicada em { $time }
composer_schedule_clear = Não agendar
composer_add_images = Adicionar imagens
composer_add_video = Adicionar vídeo
composer_add_poll = Adicionar enquete
composer_add_post = Adicionar outra publicação
composer_scheduled = Agendada para { $time }
composer_thread_sent = Sequência de { $count } publicações enviada
composer_remove_video = Remover vídeo
composer_remove_image = Remover imagem
composer_uploading = Enviando
composer_uploading_video = Enviando vídeo… { $percent }%
composer_alt_video = Descreva este vídeo
composer_alt_image = Descreva esta imagem
composer_alt_label = Descrição (texto alternativo) do anexo { $index }
composer_emoji = Adicionar emoji
poll_length_hours = { $n } h
poll_length_days = { $n } d
poll_choice = Opção { $n }
poll_choice_optional = Opção { $n } (opcional)
poll_remove_choice = Remover opção { $n }
poll_add_choice = Adicionar opção
poll_length = Duração da enquete
poll_multiple_label = Permitir várias opções
poll_remove = Remover enquete
schedule_title = Agendar publicação
schedule_hour = Em uma hora
schedule_evening = Hoje à noite
schedule_tomorrow = Amanhã de manhã
schedule_week = Em uma semana
schedule_window = Escolha um horário entre um minuto e 90 dias a partir de agora.
schedule_custom = Escolher data e hora
schedule_set = Definir
schedule_clear = Não agendar
schedule_view = Ver publicações agendadas
dropzone_drop = Solte para anexar
dropzone_reject = Não é possível usar este arquivo
nav_messages = Mensagens
nav_lists = Listas
nav_scheduled = Agendadas
notify_message = @{ $handle } enviou uma mensagem para você
shortcut_go_messages = Mensagens
shortcut_go_lists = Listas
cropper_stage = Área de corte. Arraste para mover a imagem. Use as setas do teclado para ajustar e mais e menos para ampliar.
cropper_hint = Arraste para reposicionar. Use o gesto de pinça, a rolagem ou o controle deslizante para dar zoom.
cropper_zoom = Zoom
cropper_zoom_in = Aproximar
cropper_zoom_out = Afastar
cropper_rotate = Girar
cropper_reset = Redefinir
cropper_apply = Aplicar
image_field_avatar = Foto de perfil
image_field_banner = Banner
image_field_edit_avatar = Editar foto de perfil
image_field_edit_banner = Editar banner
image_field_change = Alterar
image_field_remove = Remover
image_field_hint = PNG, JPEG, GIF ou WebP de até 10 MB. Um GIF vira imagem estática.
image_field_not_image = Esse arquivo não é uma imagem que o tuita possa usar (PNG, JPEG, GIF ou WebP).
image_field_too_large = A imagem é grande demais (máx. 10 MB).
image_field_uploading = Enviando
image_field_saved = Imagem salva.
image_field_removed = Imagem removida.
edit_title = Editar publicação
edit_note = Você pode editar por 30 minutos depois de publicar. As pessoas veem que a publicação foi editada e também as versões anteriores.
edit_save = Salvar
edit_saved = Publicação atualizada
edit_edited = Editada
edit_history_title = Histórico de edições
edit_history_current = Versão atual
edit_history_earlier = Versão anterior
edit_history_original = Original
report_post_title = Denunciar publicação
report_user_title = Denunciar { $handle }
report_intro = O que há de errado? Os moderadores desta instância vão analisar sua denúncia. { $handle } não saberá quem denunciou.
report_reason = Motivo
report_note = Mais alguma coisa? (opcional)
report_send = Enviar denúncia
report_sent = Obrigado. Sua denúncia foi enviada aos moderadores.
report_duplicate = Você já denunciou isto. Os moderadores vão dar uma olhada.
report_spam = Spam
report_spam_body = Anúncios indesejados, golpes, conteúdo repetitivo ou enganoso
report_harassment = Assédio
report_harassment_body = Perseguir, intimidar ou ameaçar alguém
report_hate = Discurso de ódio
report_hate_body = Ataca pessoas por quem elas são
report_violence = Violência ou automutilação
report_violence_body = Ameaças, glorificação da violência, incentivo à automutilação
report_misinformation = Desinformação
report_misinformation_body = Afirmações falsas que podem causar danos
report_impersonation = Falsa identidade
report_impersonation_body = Fingir ser outra pessoa
report_other = Outra coisa
report_other_body = Viola as regras de outra forma
post_edit = Editar publicação
post_edit_history = Histórico de edições
post_report = Denunciar publicação
post_hide_reply = Ocultar resposta
post_unhide_reply = Mostrar resposta
post_share_menu = Compartilhar…
post_reply_hidden = Resposta oculta
post_reply_hidden_toast = Resposta ocultada
post_reply_unhidden_toast = Resposta visível novamente
sensitive_label = Mídia sensível
sensitive_show = Mostrar
sensitive_hide = Ocultar mídia sensível
media_label =
    { $count ->
        [one] { $count } anexo
       *[other] { $count } anexos
    }
media_video = Vídeo
media_open = Abrir imagem { $index } de { $count }
lightbox_title = Visualizador de mídia
lightbox_prev = Anterior
lightbox_next = Próxima
poll_vote = Votar
poll_your_vote = Seu voto
poll_votes =
    { $count ->
        [one] { $compact } voto
       *[other] { $compact } votos
    }
poll_final = Resultados finais
poll_multiple = múltipla escolha
poll_left_minutes = { $n } min restantes
poll_left_hours = { $n } h restantes
poll_left_days = { $n } d restantes
pull_refresh = Puxe para atualizar
messages_title = Mensagens
messages_new = Nova mensagem
messages_label = Conversas
messages_empty_title = Comece uma conversa
messages_empty_body = As mensagens que você enviar e receber aparecem aqui.
messages_you_prefix = Você: { $text }
messages_unread = Não lidas: { $count }
messages_new_search = Pesquisar pessoas
messages_new_results = Pessoas
messages_new_none = Ninguém encontrado.
chat_load_older = Carregar mensagens anteriores
chat_intro = Este é o início da sua conversa.
chat_label = Mensagens com { $name }
chat_delete = Excluir
chat_seen = Visto
chat_delete_title = Excluir esta mensagem?
chat_delete_body = Ela desaparece para vocês dois.
chat_doc_title = Conversa com { $name }
chat_error_title = Não foi possível abrir esta conversa
chat_placeholder = Escreva uma mensagem
chat_input_label = Mensagem
chat_send = Enviar
lists_title = Listas
lists_new = Nova lista
lists_edit = Editar lista
lists_label = Suas listas
lists_empty_title = Nenhuma lista ainda
lists_empty_body = Agrupe pessoas em listas para ler as publicações delas separadamente.
lists_members =
    { $count ->
        [one] { $compact } membro
       *[other] { $compact } membros
    }
lists_private = Privada
lists_name = Nome
lists_description = Descrição
lists_private_switch = Lista privada
lists_save = Salvar
lists_picker_title = Adicionar { $handle } às listas
lists_picker_none = Você ainda não tem listas.
lists_delete = Excluir lista
lists_delete_title = Excluir esta lista?
lists_delete_body = “{ $name }” será removida. As pessoas nela não são afetadas.
lists_by = Lista de { $handle }
lists_tab_posts = Publicações
lists_tab_members = Membros
lists_tabs = Seções da lista
lists_error_title = Não foi possível abrir esta lista
lists_posts_label = Publicações da lista
lists_posts_empty_title = Nada aqui ainda
lists_posts_empty_body = As publicações dos membros desta lista aparecem aqui.
lists_members_label = Membros da lista
lists_members_empty_title = Nenhum membro
lists_members_empty_body = Adicione pessoas a partir dos perfis delas.
lists_remove = Remover
scheduled_title = Publicações agendadas
scheduled_label = Publicações agendadas
scheduled_status_pending = Agendada
scheduled_status_published = Publicada
scheduled_status_failed = Falhou
scheduled_cw = Aviso de conteúdo: { $text }
scheduled_poll = Enquete com { $count } opções, dura { $hours } h
scheduled_change_time = Alterar horário
scheduled_post_now = Publicar agora
scheduled_remove = Remover
scheduled_window_error = Escolha um horário entre um minuto e 90 dias a partir de agora.
scheduled_when = Publicar em
scheduled_zone = Fuso horário: { $zone }
scheduled_save = Salvar
scheduled_posted = Publicada.
scheduled_cancel_title = Cancelar esta publicação agendada?
scheduled_cancel_body = Ela não será publicada.
scheduled_cancel_confirm = Descartar publicação
scheduled_empty_title = Nada agendado
scheduled_empty_body = As publicações que você agendar esperam aqui até a hora certa.
profile_message = Mensagem
profile_add_to_lists = Adicionar a listas
media_alt = ALT
profile_report = Denunciar { $handle }
chart_show_table = Mostrar { $title } como tabela
chart_col_day = Dia
chart_col_hour = Hora (UTC)
chart_empty = { $title }: nada para mostrar ainda.
chart_bar_summary = { $title }: { $total } no total, mais em { $label } ({ $peak }).
chart_spark_summary = { $title }: { $total } no total, { $last } em { $label }.
chart_heat_summary = { $title }: { $total } no total, pico em { $label } às { $hour } ({ $peak }).
chart_heat_less = Menos
chart_heat_more = Mais
announce_region = Avisos
announce_dismiss = Dispensar aviso
announce_more = Ler mais
announce_tone_info = Informação
announce_tone_success = Boa notícia
announce_tone_warning = Atenção
admin_stat_people = Pessoas
admin_stat_people_sub = Ativas esta semana: { $count }
admin_stat_posts_today = Publicações, 24 h
admin_stat_posts_total = { $count } no total
admin_stat_reports = Denúncias abertas
admin_stat_reports_open = Precisa de análise
admin_stat_reports_clear = Tudo em ordem
admin_stat_signups_today = Cadastros hoje
admin_stat_signups_sub = { $count } em 14 dias
admin_stat_suspended = Suspensas
admin_stat_suspended_sub = contas
admin_chart_posts = Publicações por dia
admin_chart_signups = Cadastros por dia
admin_chart_reports = Denúncias por dia
admin_chart_heat = Atividade por dia da semana e hora
nav_admin = Administração
admin_title = Administração
admin_sections = Seções da administração
admin_tab_overview = Visão geral
admin_tab_reports = Denúncias
admin_tab_users = Pessoas
admin_tab_instance = Instância
admin_tab_federation = Federação
admin_tab_announcements = Avisos
admin_tab_invites = Convites
admin_tab_filter = Filtro de conteúdo
admin_tab_audit = Registro de auditoria
admin_denied_title = Só para a equipe
admin_denied_body = Esta área é para os administradores e moderadores desta instância.
admin_denied_home = Ir para o início
admin_unknown = desconhecido
admin_tag_active = Ativa
admin_tag_suspended = Suspensa
admin_tag_deleted = Excluída
admin_tag_org = Organização
admin_audit_system = sistema
admin_audit_empty = Nada registrado ainda
admin_report_status = Status da denúncia
admin_report_open = Abertas
admin_report_resolved = Resolvidas
admin_reports_empty_open = Nenhuma denúncia aberta
admin_reports_empty_open_body = Quando as pessoas denunciarem publicações ou contas, elas aparecerão aqui.
admin_reports_empty_resolved = Nada resolvido ainda
admin_report_by = denunciado por { $handle }
admin_report_same_target = { $count } denúncias abertas
admin_report_media =
    { $count ->
        [one] { $count } anexo
       *[other] { $count } anexos
    }
admin_report_poll = Tem uma enquete
admin_report_dismiss = Dispensar
admin_report_delete_post = Excluir publicação
admin_report_suspend = Suspender conta
admin_report_delete_suspend = Excluir e suspender
admin_report_confirm_title = Confirmar esta ação?
admin_report_confirm_body = { $action }. Isso fica anotado no registro de auditoria.
admin_report_confirm = Confirmar
admin_report_resolution = { $action } por { $handle } · { $time }
admin_res_dismiss = Dispensada
admin_res_delete_post = Publicação excluída
admin_res_suspend_user = Conta suspensa
admin_res_delete_and_suspend = Publicação excluída e conta suspensa
admin_users_search = Pesquisar pessoas
admin_users_search_placeholder = Nome ou nome de usuário
admin_users_show = Mostrar
admin_users_empty = Ninguém encontrado
admin_filter_local = Locais
admin_filter_remote = Remotas
admin_filter_staff = Equipe
admin_filter_verified = Verificadas
admin_filter_suspended = Suspensas
admin_filter_all = Todas
admin_user_details = Detalhes de { $handle }
admin_user_menu = Gerenciar { $handle }
admin_user_posts =
    { $count ->
        [one] { $compact } publicação
       *[other] { $compact } publicações
    }
admin_user_followers =
    { $count ->
        [one] { $compact } seguidor
       *[other] { $compact } seguidores
    }
admin_user_joined = entrou: { $time }
admin_user_active = ativo: { $time }
admin_user_open_reports =
    { $count ->
        [one] { $compact } denúncia aberta
       *[other] { $compact } denúncias abertas
    }
admin_act_verify = Verificar
admin_act_unverify = Remover verificação
admin_act_org = Tornar organização
admin_act_unorg = Remover status de organização
admin_act_role_user = Remover função da equipe
admin_act_role_moderator = Tornar moderador
admin_act_role_admin = Tornar administrador
admin_act_suspend = Suspender
admin_act_unsuspend = Reativar
admin_act_purge = Excluir todas as publicações
admin_act_rename = Alterar nome de usuário
admin_act_reset_name = Redefinir nome de exibição
admin_act_clear_bio = Limpar bio
admin_act_delete = Excluir conta
admin_done_verified = { $handle } está verificado
admin_done_unverified = Verificação removida
admin_done_org = { $handle } agora é uma organização
admin_done_unorg = Status de organização removido
admin_done_role = Função atualizada
admin_done_suspended = { $handle } suspenso
admin_done_unsuspended = { $handle } reativado
admin_done_purged =
    { $count ->
        [one] { $count } publicação excluída
       *[other] { $count } publicações excluídas
    }
admin_done_name_reset = Nome de exibição redefinido
admin_done_bio_cleared = Bio limpa
admin_done_deleted = { $handle } excluído
admin_done_renamed = Agora é { $handle }
admin_role_phrase_user = um usuário comum
admin_role_phrase_moderator = um moderador
admin_role_phrase_admin = um administrador
admin_role_title = Alterar a função de { $handle }?
admin_role_body = A pessoa passará a ser { $role }.
admin_role_confirm = Alterar função
admin_unorg_title = Remover o status de organização de { $handle }?
admin_unorg_body = Todas as afiliações dela terminam.
admin_unorg_confirm = Remover
admin_suspend_title = Suspender { $handle }?
admin_suspend_body = A pessoa será desconectada e o conteúdo dela ficará oculto até você reativá-la.
admin_suspend_confirm = Suspender
admin_purge_title = Excluir todas as publicações de { $handle }?
admin_purge_body = Isso não pode ser desfeito.
admin_purge_confirm = Excluir tudo
admin_reset_name_title = Redefinir o nome de exibição de { $handle }?
admin_reset_name_body = Ele será substituído pelo nome de usuário.
admin_reset_name_confirm = Redefinir
admin_delete_title = Excluir { $handle }?
admin_delete_body = As publicações, seguidores e mensagens dela são removidos e a conta é anonimizada. Isso não pode ser desfeito.
admin_delete_confirm = Excluir conta
admin_rename_title = Alterar o nome de usuário de { $handle }
admin_rename_note = Os links para o nome de usuário antigo deixam de funcionar.
admin_rename_handle = Novo nome de usuário
admin_rename_reset_name = Redefinir também o nome de exibição
admin_rename_save = Alterar nome de usuário
admin_detail_title = Detalhes da conta
admin_detail_invite = Convite { $code }
admin_detail_posts = Publicações
admin_detail_followers = Seguidores
admin_detail_following = Seguindo
admin_detail_open_reports = Denúncias abertas
admin_detail_sessions = Sessões
admin_detail_passkeys = Chaves de acesso
admin_detail_tokens = Tokens de API
admin_detail_joined = Entrou em { $time }
admin_detail_recent_posts = Publicações recentes
admin_detail_no_posts = Nenhuma publicação.
admin_detail_reports = Denúncias sobre esta conta
admin_detail_no_reports = Nenhuma denúncia.
admin_detail_history = Histórico de moderação
admin_detail_nothing = Nada ainda.
admin_instance_title = Instância
admin_instance_name = Nome da instância
admin_instance_description = Descrição
admin_instance_contact = E-mail de contato
admin_instance_contact_hint = Mostrado nas páginas de Termos e Privacidade
admin_instance_min_age = Idade mínima
admin_instance_rules = Regras
admin_instance_rules_hint = Uma regra por linha
admin_instance_preview = Ver prévia dos termos
admin_instance_save = Salvar
admin_instance_saved = Configurações da instância salvas
admin_instance_template_note = Os Termos de Serviço e a Política de Privacidade são modelos que usam estes valores. Peça uma revisão para a sua jurisdição antes de abrir a instância ao público.
admin_signups_title = Cadastros
admin_signups_desc = Quem pode criar uma conta nesta instância
admin_signups_open = Abertos
admin_signups_open_hint = Qualquer pessoa pode criar uma conta.
admin_signups_invite = Só com convite
admin_signups_invite_hint = As pessoas precisam de um código de convite da aba Convites.
admin_signups_closed = Fechados
admin_signups_closed_hint = Ninguém pode se cadastrar.
admin_filter_title = Filtro de conteúdo
admin_filter_desc = Nada é bloqueado até você adicionar palavras abaixo. As palavras bloqueadas valem para nomes de usuário, nomes de exibição, localizações, bios, publicações e nomes de listas, inclusive grafias disfarçadas. Nomes de usuário reservados, como “admin”, não podem ser registrados.
admin_filter_blocked = Palavras bloqueadas
admin_filter_blocked_hint = Uma por linha, encontradas também dentro de palavras
admin_filter_allowed = Palavras permitidas
admin_filter_allowed_hint = Exceções às palavras bloqueadas, uma por linha
admin_filter_reserved = Nomes de usuário reservados extras
admin_filter_reserved_hint = Um por linha
admin_filter_save = Salvar filtro
admin_filter_saved = Filtro salvo
admin_filter_field_handle = nome de usuário
admin_filter_field_name = nome
admin_filter_field_location = localização
admin_filter_field_bio = bio
admin_scan_title = Verificar contas
admin_scan_desc = Encontre contas cujo nome de usuário, nome ou bio violem o filtro.
admin_scan_run = Verificar agora
admin_scan_clean = Nenhuma conta viola o filtro.
admin_scan_found =
    { $count ->
        [one] { $count } conta precisa de atenção
       *[other] { $count } contas precisam de atenção
    }
admin_scan_reason = O campo { $field } contém { $reason }
admin_ann_title = Novo aviso
admin_ann_desc = Um banner no topo da tela de todos até expirar ou você encerrá-lo. As pessoas podem dispensá-lo.
admin_ann_text = Mensagem
admin_ann_link = Link
admin_ann_link_hint = Opcional: https://… ou /caminho
admin_ann_tone = Estilo
admin_ann_tone_info = Informação
admin_ann_tone_success = Boa notícia
admin_ann_tone_warning = Atenção
admin_ann_duration = Duração
admin_ann_days =
    { $count ->
        [one] { $count } dia
       *[other] { $count } dias
    }
admin_ann_week = 1 semana
admin_ann_until_ended = Até encerrar
admin_ann_publish = Publicar
admin_ann_published = Aviso publicado
admin_ann_list = Avisos até agora
admin_ann_none = Nenhum aviso ainda.
admin_ann_end = Encerrar
admin_ann_ended = encerrado
admin_ann_expired = expirado
admin_ann_inactive = Não exibido
admin_ann_until = até { $date }
admin_inv_title = Códigos de convite
admin_inv_desc = Permita que pessoas específicas entrem enquanto os cadastros são só por convite.
admin_inv_note = Nota (para quem é)
admin_inv_uses = Usos
admin_inv_use_count =
    { $count ->
        [one] { $count } uso
       *[other] { $count } usos
    }
admin_inv_expiry = Expira
admin_inv_week = Em 1 semana
admin_inv_days =
    { $count ->
        [one] Em { $count } dia
       *[other] Em { $count } dias
    }
admin_inv_never = Nunca
admin_inv_create = Criar código
admin_inv_created = Código { $code } criado
admin_inv_created_copied = Código { $code } criado e copiado
admin_inv_list = Códigos
admin_inv_none = Nenhum código de convite ainda.
admin_inv_state_active = Ativo
admin_inv_state_revoked = Revogado
admin_inv_state_used = Esgotado
admin_inv_state_expired = Expirado
admin_inv_used = { $uses } de { $max } usados
admin_inv_expires = expira em { $date }
admin_inv_copy = Copiar código { $code }
admin_inv_copied = Código copiado
admin_inv_copy_failed = Não foi possível copiar. O código é { $code }.
admin_inv_revoke = Revogar
admin_inv_revoke_title = Revogar este convite?
admin_inv_revoke_body = As pessoas que já entraram com ele mantêm as contas.
settings_devices = Dispositivos e sessões
settings_devices_desc = Veja onde você está conectado e saia remotamente
settings_passkeys = Chaves de acesso
settings_passkeys_desc = Entre com sua digital, rosto, bloqueio de tela ou uma chave de segurança
settings_affiliation = Afiliação
settings_affiliation_desc = Organizações das quais você faz parte ou as contas da sua organização
settings_bots = Bots e API
settings_bots_desc = Tokens de API e o selo “Automatizada”
settings_saved = Salvo
passkey_title = Suas chaves de acesso
passkey_desc = Entre com sua digital, rosto, bloqueio de tela ou chave de segurança em vez de uma senha. As chaves de acesso não podem ser alvo de phishing e nada secreto fica guardado no nosso servidor.
passkey_none = Ainda não há chaves de acesso.
passkey_add = Adicionar chave de acesso
passkey_added = Chave de acesso “{ $name }” adicionada
passkey_default_name = Chave de acesso
passkey_added_ago = adicionada: { $time }
passkey_used_ago = último uso: { $time }
passkey_never_used = nunca usada
passkey_rename = Renomear
passkey_rename_label = Renomear chave de acesso { $name }
passkey_rename_title = Renomear chave de acesso
passkey_name = Nome
passkey_rename_save = Salvar
passkey_remove = Remover
passkey_remove_label = Remover chave de acesso { $name }
passkey_remove_title = Remover “{ $name }”?
passkey_remove_body = Você não poderá mais entrar com esta chave de acesso. Talvez também queira excluí-la do seu dispositivo ou gerenciador de senhas.
passkey_removed = Chave de acesso removida
passkey_unsupported = Este navegador não pode usar chaves de acesso neste endereço.
passkey_needs_domain = As chaves de acesso exigem que este site tenha um nome de domínio, e não um endereço IP.
passkey_already_here = Este dispositivo já tem uma chave de acesso para esta conta.
token_scope_read = Leitura
token_scope_write = Publicar e interagir
token_scope_messages = Mensagens diretas
token_scope_read_hint = Linhas do tempo, notificações, perfis, pesquisa e o fluxo em tempo real
token_scope_write_hint = Publicar, excluir, curtir, repostar, salvar, seguir e enviar arquivos
token_scope_messages_hint = Ler e enviar mensagens diretas
token_title = Tokens de API
token_desc = Tokens permitem que scripts e bots usem sua conta pela API. Trate-os como senhas: quem tiver um pode agir como você dentro das permissões dele.
token_none = Ainda não há tokens.
token_name = Nome do token
token_name_hint = Por exemplo: bot do tempo
token_scopes = Escopos
token_create = Criar token
token_used_ago = último uso: { $time }
token_never_used = nunca usado
token_revoke = Revogar
token_revoke_label = Revogar token { $name }
token_revoke_title = Revogar “{ $name }”?
token_revoke_body = Tudo o que o usa deixa de funcionar.
token_revoked = Token revogado
token_secret_title = Seu novo token
token_secret_warning = Copie agora. Por segurança, ele não será mostrado de novo.
token_secret_label = Novo token de API
token_copy = Copiar
token_copied = Token copiado
token_copy_failed = Não foi possível copiar. Selecione o token e copie manualmente.
token_done = Concluído
bot_title = Conta automatizada
bot_desc = Mostra o selo “Automatizada” ao lado do nome, para todos saberem que um bot está publicando.
bot_switch = Esta é uma conta automatizada
bot_owner = Administrado por
bot_owner_hint = O nome de usuário da pessoa ou organização responsável. Opcional.
bot_owner_save = Salvar
bot_docs = Documentação para desenvolvedores
bot_docs_desc = Endpoints, eventos em tempo real e bots de exemplo
aff_yours_title = Sua afiliação
aff_yours_desc = Uma organização pode convidar você a mostrar o selo dela ao lado do seu nome. Nada muda até você aceitar, e você pode sair quando quiser.
aff_none = Você não está afiliado a nenhuma organização.
aff_invites_title = Convites
aff_accept = Aceitar
aff_decline = Recusar
aff_leave = Sair
aff_leave_title = Sair desta organização?
aff_leave_body = O selo dela sai do seu perfil.
aff_remove_title = Remover { $handle }?
aff_remove_body = Seu selo sai do perfil da pessoa.
aff_remove = Remover
aff_remove_label = Remover { $handle }
aff_ended = Afiliação encerrada
aff_declined = Convite recusado
aff_accepted = Afiliação adicionada
aff_invited = Convite enviado
aff_org_title = Seus afiliados
aff_org_desc = Convide pessoas para exibir seu selo ao lado do nome delas. Elas precisam aceitar.
aff_handle = Nome de usuário
aff_label = Rótulo
aff_label_hint = Opcional, como “Engenheiro”
aff_invite = Enviar convite
aff_members_none = Nenhum afiliado ainda.
aff_pending = Convidado
aff_not_org = Você é uma organização? Peça a um administrador desta instância para marcar sua conta.
auth_passkey = Entrar com uma chave de acesso
auth_or = ou
auth_invite = Código de convite
auth_invite_hint = Esta instância só aceita novas pessoas por convite.
auth_invite_required = Digite o código de convite que você recebeu.
landing_developers = Desenvolvedores
landing_invite = Os cadastros são só por convite. Tem um código? Crie sua conta.
dev_title = Desenvolvedores
dev_hero = Bots e API
dev_hero_sub = Crie bots, scripts e integrações para o { $name }
dev_intro_h = Crie para o { $name }
dev_intro_p = Tudo o que os apps do tuita fazem passa por uma pequena API JSON, e bots e scripts também podem usá-la. Você precisa de uma conta no { $name } e de um token de API pessoal. Não há registro de app nem etapas de OAuth.
dev_intro_1 = Crie uma conta para o seu bot ou use a sua.
dev_intro_2 = Em Configurações, Bots e API, ative “Esta é uma conta automatizada” para que as pessoas vejam o selo “Automatizada”, e informe quem a administra.
dev_intro_3 = Crie um token com os escopos de que precisa e mantenha-o em segredo.
dev_auth_h = Autenticação
dev_auth_p = Envie o token no cabeçalho Authorization. Toda requisição e resposta é JSON (UTF-8), exceto os envios de arquivos, que são multipart.
dev_auth_prefix = Um token é tuita_ seguido de 43 letras, dígitos, hifens e sublinhados. Só um hash é guardado, então um token perdido não pode ser mostrado de novo: crie outro. Um token tem um ou mais destes escopos:
dev_auth_refused = Um token não pode alterar senhas, sessões, tokens ou chaves de acesso, não pode exportar nem excluir a conta e não pode usar a API de administração. Essas ações exigem entrar de verdade pelo site; pela API, elas respondem 403.
dev_auth_errors = Os erros voltam como JSON com uma mensagem de erro e um status 4xx. 401 significa que o token está errado ou foi revogado, 403 que falta um escopo ou a rota foi recusada, e 429 vem com um cabeçalho Retry-After.
dev_post_h = Sua primeira publicação
dev_post_p = Envie imagens e vídeo primeiro com POST /api/v1/media (multipart) e depois anexe-os pelo id em mediaIds. Uma resposta define replyTo, uma citação define quoteOf.
dev_endpoints_h = Endpoints
dev_endpoints_intro = Os trechos entre chaves nos caminhos são marcadores de posição. As listas paginam com ?before= definido como o próximo valor da página anterior; limit vai de 1 a 40. A coluna de escopo mostra o escopo de que um token precisa.
dev_col_method = Método
dev_col_path = Caminho
dev_col_scope = Escopo
dev_col_what = O que faz
dev_ep_me = A conta à qual o token pertence
dev_ep_patch_me = Altera o perfil e as configurações dessa conta
dev_ep_bot = Ativa ou desativa o selo “Automatizada” e informa quem administra o bot
dev_ep_following = A linha do tempo principal, com as pessoas que você segue
dev_ep_foryou = A linha do tempo Para você; pagine com ?cursor=
dev_ep_notifications = Notificações (?filter=mentions para respostas, menções e citações)
dev_ep_post = Uma publicação com sua conversa e respostas
dev_ep_user = Um perfil (usuario@dominio para contas de outras instâncias)
dev_ep_user_posts = Publicações de alguém (?tab=replies, media ou likes; com media também ?kind=photo ou video)
dev_ep_search = Pesquisa publicações, pessoas ou mídia, com operadores como from: e has:media
dev_ep_trends = As tags mais usadas na última semana
dev_ep_stream = Eventos em tempo real (Server-Sent Events)
dev_ep_create = Escreve uma publicação
dev_ep_thread = Escreve de 2 a 25 publicações como uma sequência
dev_ep_delete = Exclui uma das suas publicações
dev_ep_like = Curte uma publicação (DELETE para desfazer). Também /repost e /bookmark
dev_ep_follow = Segue alguém (DELETE para deixar de seguir)
dev_ep_media = Envia uma imagem ou vídeo como multipart; a resposta traz o id para anexar
dev_ep_schedule = Agenda uma publicação para mais tarde
dev_ep_report = Denuncia uma publicação ou conta aos moderadores
dev_ep_conversations = Suas conversas de mensagens diretas
dev_ep_message = Envia uma mensagem direta; o corpo é JSON com um campo text
dev_realtime_h = Eventos em tempo real
dev_realtime_p = GET /api/v1/stream é um fluxo de Server-Sent Events. Ele envia um evento notification para cada curtida, repostagem, novo seguidor, resposta, menção ou citação, um evento message para mensagens diretas e badges com as contagens de não lidas. Envia uma linha de comentário a cada 25 segundos e pede aos clientes que tentem de novo após 5 segundos; reconecte se a conexão cair.
dev_example_h = Exemplo: um bot que responde a menções
dev_example_node = Node 22 ou mais recente, sem pacotes:
dev_example_python = O mesmo em Python, usando apenas requests:
dev_limits_h = Limites
dev_limit_rate = Cada token pode fazer 600 requisições a cada 10 minutos. Acima do limite, o servidor responde 429 com um cabeçalho Retry-After.
dev_limit_tokens = Uma conta pode ter até 20 tokens e criar até 20 tokens por hora.
dev_limit_posting = Publicar, seguir, denunciar e as outras ações têm limites próprios por conta.
dev_rules_h = Regras para bots
dev_rule_1 = Ative o selo Automatizada e informe quem administra o bot.
dev_rule_2 = Só responda ou envie mensagens a quem pediu: quem mencionou o bot, o seguiu ou mandou mensagem primeiro. Não envie menções ou mensagens diretas não solicitadas.
dev_rule_3 = Vá com calma: fique bem abaixo dos limites acima.
dev_rule_4 = Siga os Termos e as regras da instância. Os moderadores podem suspender bots como qualquer conta.
admin_act_log_admin_bootstrap = Primeiro administrador configurado
admin_act_log_announcement_create = Aviso publicado
admin_act_log_announcement_end = Aviso encerrado
admin_act_log_instance_settings = Configurações da instância alteradas
admin_act_log_invite_create = Convite criado
admin_act_log_invite_revoke = Convite revogado
admin_act_log_post_delete = Publicação excluída
admin_act_log_report_dismiss = Denúncia dispensada
admin_act_log_user_delete = Conta excluída
admin_act_log_user_org = Marcada como organização
admin_act_log_user_unorg = Status de organização removido
admin_act_log_user_purge_posts = Todas as publicações excluídas
admin_act_log_user_rename = Conta renomeada
admin_act_log_user_role = Função alterada
admin_act_log_user_suspend = Conta suspensa
admin_act_log_user_unsuspend = Conta reativada
admin_act_log_user_unverify = Verificação removida
admin_act_log_user_verify = Conta verificada
admin_act_log_domain_block = Instância bloqueada
admin_act_log_domain_unblock = Instância desbloqueada
admin_act_log_federation_retry = Nova tentativa de entrega solicitada
admin_fed_stat_domains = Instâncias conhecidas
admin_fed_stat_domains_sub = vistas por esta instância
admin_fed_stat_accounts = Contas remotas
admin_fed_stat_accounts_sub = conhecidas de outros lugares
admin_fed_stat_posts = Publicações remotas
admin_fed_stat_posts_sub = armazenadas localmente
admin_fed_stat_queue = Na fila
admin_fed_stat_queue_sub = entregas aguardando
admin_fed_queue_title = Fila de entrega
admin_fed_queue_desc = Atividades assinadas aguardando para chegar a outras instâncias. As que falham são tentadas de novo com intervalos crescentes e descartadas após 12 tentativas.
admin_fed_queue_empty = Nada aguardando. Todas as entregas foram enviadas.
admin_fed_queue_waiting =
    { $count ->
        [one] { $compact } entrega aguardando
       *[other] { $compact } entregas aguardando
    }
admin_fed_queue_due = Prontas para envio: { $count }
admin_fed_queue_oldest = Mais antiga na fila: { $date }
admin_fed_retry = Tentar agora
admin_fed_retry_done =
    { $count ->
        [one] { $compact } entrega será tentada de novo agora.
       *[other] { $compact } entregas serão tentadas de novo agora.
    }
admin_fed_failing = Instâncias com falha
admin_fed_failing_none = Nenhuma instância com falha.
admin_fed_failing_count =
    { $count ->
        [one] { $compact } entrega
       *[other] { $compact } entregas
    }
admin_fed_failing_attempts =
    { $count ->
        [one] { $compact } tentativa
       *[other] { $compact } tentativas
    }
admin_fed_failing_next = próxima tentativa em { $date }
admin_fed_failing_error = Último erro
admin_fed_failing_no_error = Nenhum erro registrado
admin_fed_actor_title = Ator da instância
admin_fed_actor_desc = As outras instâncias verificam com esta chave as requisições que esta instância assina.
admin_fed_actor_id = Ator
admin_fed_actor_key = ID da chave
admin_fed_actor_fingerprint = Impressão digital SHA-256
admin_fed_actor_copy = Copiar impressão digital
admin_fed_actor_copied = Impressão digital copiada
admin_fed_actor_copy_failed = Não foi possível copiar. A impressão digital é { $fingerprint }
admin_fed_blocks_title = Instâncias bloqueadas
admin_fed_blocks_desc = Silenciar oculta as contas de uma instância das linhas do tempo públicas e da pesquisa. Suspender oculta em todo lugar, descarta o que ela envia e interrompe as entregas para ela.
admin_fed_domain = Domínio
admin_fed_domain_hint = example.social ou um link para qualquer página dela
admin_fed_severity = Gravidade
admin_fed_silence = Silenciar
admin_fed_suspend = Suspender
admin_fed_reason = Motivo
admin_fed_reason_hint = Opcional, mostrado apenas à equipe
admin_fed_block = Bloquear instância
admin_fed_blocked = { $domain } está bloqueada
admin_fed_none = Nenhuma instância bloqueada.
admin_fed_list = Bloqueios
admin_fed_row_accounts =
    { $count ->
        [one] { $compact } conta conhecida
       *[other] { $compact } contas conhecidas
    }
admin_fed_row_no_reason = Nenhum motivo informado
admin_fed_row_changed = Gravidade de { $domain } alterada para { $severity }
admin_fed_remove = Remover
admin_fed_remove_label = Remover o bloqueio de { $domain }
admin_fed_remove_title = Remover o bloqueio de { $domain }?
admin_fed_remove_body = As contas e publicações dela voltam e as entregas para ela recomeçam.
admin_fed_removed = { $domain } não está mais bloqueada
remote_domain_title = Conta em { $domain }
remote_view_original = Ver em { $domain }
visibility_unlisted = Não listada
visibility_unlisted_hint = Não aparece nas linhas do tempo públicas
visibility_followers = Só seguidores
visibility_followers_hint = Só os seguidores podem ver isto
visibility_direct = Direta
visibility_direct_hint = Só as pessoas mencionadas podem ver isto
follow_requested = Solicitado
follow_cancel_request = Cancelar pedido para seguir
follow_requested_toast = Pedido para seguir enviado a { $handle }
follow_cancelled_toast = Pedido para seguir cancelado
profile_locked = Aprova seguidores
profile_locked_hint = Os pedidos para seguir precisam da aprovação desta conta.
profile_original = Ver perfil em { $domain }
profile_remote_note = Uma cópia de { $domain }. As contagens e publicações são as que esta instância viu.
remote_lookup_loading = Procurando { $handle } em { $domain }…
remote_lookup_notfound_title = Conta não encontrada
remote_lookup_notfound_body = { $handle } não existe ou { $domain } não a compartilha.
remote_lookup_failed_title = Não foi possível carregar { $handle }
explore_lookup = Procurando { $target }…
explore_lookup_empty_body = Essa conta ou publicação não foi encontrada. A outra instância pode estar fora do ar ou não compartilhá-la.
language_label = Idioma
oauth_title = Autorizar um app
oauth_loading = Verificando a solicitação…
oauth_wants = Este app quer usar sua conta { $handle }.
oauth_name_warning = Qualquer pessoa pode escolher qualquer nome de app. Aprove apenas apps em que você confia.
oauth_website = Site
oauth_allow = Ele poderá
oauth_scope_read = Ler seu perfil, publicações, seguidores, listas, linhas do tempo e notificações, e as mensagens diretas que não têm criptografia de ponta a ponta.
oauth_scope_write = Publicar, editar e excluir por você, curtir, seguir, bloquear, silenciar e alterar seu perfil e configurações.
oauth_scope_messages = Ler e enviar suas mensagens diretas.
oauth_push_note = As notificações push não estão disponíveis. Um app que as solicitar não as receberá.
oauth_redirect = Depois de responder, você será enviado para
oauth_redirect_oob = O app não recebe redirecionamento. Um código é mostrado aqui para você colar nele.
oauth_approve = Autorizar
oauth_deny = Negar
oauth_switch = Usar outra conta
oauth_sent_approved = Autorizado. Levando você de volta a { $host }. Se nada acontecer, feche esta aba.
oauth_sent_denied = Negado. Levando você de volta a { $host }. Se nada acontecer, feche esta aba.
oauth_denied = Negado. Nada foi compartilhado com o app.
oauth_code_title = Código de autorização
oauth_code_body = Cole este código no app. Ele funciona uma vez e expira em 10 minutos.
oauth_code_copy = Copiar código
oauth_code_copied = Código copiado
oauth_code_copy_failed = Não foi possível copiar. Selecione o código e copie manualmente.
oauth_error_title = Esta solicitação não pode ser autorizada
oauth_error_note = Nada foi enviado ao app.
oauth_error_invalid_request = A solicitação está malformada ou pede para voltar a um endereço que o app não registrou. Volte ao app e tente de novo.
oauth_error_invalid_client = Este app não está registrado aqui.
oauth_error_invalid_scope = O app pede um acesso que o tuita não oferece ou mais do que registrou.
oauth_error_unsupported_response_type = O app pede um tipo de login que o tuita não aceita.
oauth_error_rate = Você respondeu a solicitações demais. Aguarde alguns minutos e tente de novo.
oauth_error_unreachable = Não foi possível acessar o tuita. Verifique sua conexão e tente de novo.
oauth_home = Ir para o tuita
share_send_chat = Enviar por chat
share_download = Baixar mídia
share_download_n = Baixar mídia { $number }
share_chat_title = Enviar publicação por chat
share_chat_sent = Enviado para { $handle }
hover_card_label = Cartão de perfil de { $name }
profile_affiliation_role = ({ $label })
profile_media_filter = Tipo de mídia
profile_media_all = Todas as mídias
profile_media_photos = Fotos
profile_media_videos = Vídeos
profile_empty_videos_title = Nenhum vídeo ainda
profile_empty_videos_body = Publicações com vídeos aparecerão aqui.
account_add_existing = Adicionar uma conta existente
rail_label = Pesquisa e sugestões
rail_search_submit = Pesquisar
rail_show_more = Mostrar mais
rail_footer = Links úteis
chat_new = Novo chat
chat_search = Pesquisar conversas
chat_search_none = Nenhuma conversa corresponde à sua pesquisa.
chat_start_title = Iniciar conversa
chat_start_body = Escolha uma conversa da lista ou inicie uma nova.
explore_sections = Seções do Explorar
explore_see_more = Mostrar mais
history_title = Histórico
history_tabs = Seções do histórico
history_likes_label = Publicações curtidas
video_play = Reproduzir
video_pause = Pausar
video_mute = Silenciar
video_unmute = Ativar som
video_volume = Volume
video_seek = Avançar ou voltar
video_position = { $current } de { $total }
video_speed = Velocidade de reprodução
video_fullscreen = Tela cheia
video_exit_fullscreen = Sair da tela cheia
video_pip = Picture in picture
video_exit_pip = Sair do picture in picture
video_error = Não foi possível reproduzir este vídeo.
media_pager = { $index }/{ $count }
lightbox_hide_panel = Ocultar painel
lightbox_show_panel = Mostrar painel
lightbox_view_post = Ver publicação
lightbox_panel = Publicação e respostas
emotepick_tabs = Conjuntos de emojis
emotepick_tab_emoji = Padrão
emotepick_tab_custom = Personalizados
admin_emote_tab = Emotes
admin_emote_title = Emotes personalizados
admin_emote_desc = Os emotes pertencem a toda a instância. Qualquer pessoa pode usá-los como :shortcode: em publicações, nomes e mensagens. Um shortcode não pode ser renomeado; exclua o emote e envie-o de novo.
admin_emote_shortcode = Shortcode
admin_emote_shortcode_hint = De 2 a 32 letras, números ou sublinhados. Exibido como :shortcode:.
admin_emote_invalid = Use apenas letras, números e sublinhados, de 2 a 32 caracteres.
admin_emote_category = Categoria
admin_emote_category_hint = Opcional. Agrupa os emotes no seletor.
admin_emote_file = Imagem do emote
admin_emote_file_hint = PNG, JPEG, WebP ou GIF, até 256 KB. Solte um arquivo aqui ou escolha um.
admin_emote_file_type = Esse arquivo não é uma imagem PNG, JPEG, WebP ou GIF.
admin_emote_file_size = A imagem é grande demais (máx. 256 KB).
admin_emote_choose_file = Escolher imagem
admin_emote_change_file = Alterar imagem
admin_emote_upload = Enviar emote
admin_emote_upload_failed = Não foi possível enviar o emote. Tente novamente.
admin_emote_uploaded = :{ $shortcode }: adicionado.
admin_emote_list = Emotes desta instância
admin_emote_none = Ainda não há emotes personalizados.
admin_emote_no_category = Sem categoria
admin_emote_category_for = Categoria de :{ $shortcode }:
admin_emote_saved = Categoria salva.
admin_emote_delete = Excluir :{ $shortcode }:
admin_emote_delete_title = Excluir este emote?
admin_emote_delete_body = :{ $shortcode }: deixa de ser exibido como imagem em todo lugar e aparece como texto simples. As publicações que o usavam mantêm o texto.
admin_emote_delete_confirm = Excluir emote
admin_emote_deleted = Emote excluído.
admin_act_log_emoji_create = Emote adicionado
admin_act_log_emoji_update = Emote alterado
admin_act_log_emoji_delete = Emote excluído
e2ee_settings = Mensagens criptografadas
e2ee_settings_desc = Frase secreta, este dispositivo e redefinição
e2ee_status_label = Estado
e2ee_status_none = Desativado. Ative para enviar e ler mensagens com criptografia de ponta a ponta.
e2ee_status_locked = Bloqueado neste dispositivo. Digite sua frase secreta para ler suas mensagens aqui.
e2ee_status_ready = Ativado e desbloqueado neste dispositivo.
e2ee_keys_error = Não foi possível verificar suas chaves de criptografia.
e2ee_setup_title = Ativar mensagens criptografadas
e2ee_setup_body = As mensagens para pessoas deste servidor são criptografadas no seu navegador. O servidor guarda apenas texto que não consegue ler. Uma frase secreta permite desbloquear suas mensagens em outro dispositivo.
e2ee_setup_warning = Sua frase secreta não pode ser recuperada. Se você a esquecer, poderá redefinir a criptografia, mas suas mensagens anteriores serão perdidas.
e2ee_setup_submit = Ativar
e2ee_setup_done = As mensagens criptografadas estão ativadas
e2ee_unlock_title = Desbloquear mensagens criptografadas
e2ee_unlock_body = Digite sua frase secreta para ler e enviar mensagens criptografadas neste dispositivo.
e2ee_unlock_submit = Desbloquear
e2ee_unlock_wrong = Essa frase secreta está incorreta.
e2ee_unlock_done = Desbloqueado
e2ee_passphrase = Frase secreta
e2ee_passphrase_confirm = Confirme a frase secreta
e2ee_passphrase_hint = Pelo menos 10 caracteres.
e2ee_passphrase_short = Use pelo menos 10 caracteres.
e2ee_passphrase_mismatch = As frases secretas não coincidem.
e2ee_failed = Algo deu errado. Tente novamente.
e2ee_unsupported = Este navegador não oferece suporte a mensagens criptografadas.
e2ee_forget = Esquecer neste dispositivo
e2ee_forget_hint = Remove deste navegador sua chave e os contatos em que você confia. Sua frase secreta os desbloqueia de novo.
e2ee_forget_done = Esquecido neste dispositivo
e2ee_reset = Redefinir criptografia
e2ee_reset_hint = Cria uma nova chave. As mensagens que você enviou e recebeu antes ficam ilegíveis.
e2ee_reset_title = Redefinir a criptografia?
e2ee_reset_body = Suas mensagens anteriores ficam ilegíveis, para você e para as pessoas com quem você conversou, e não podem ser recuperadas. Seus contatos serão convidados a confiar na sua nova chave. Escolha uma nova frase secreta.
e2ee_reset_submit = Redefinir
e2ee_reset_done = A criptografia foi redefinida
e2ee_note_sealed = Criptografia de ponta a ponta
e2ee_note_remote = Sem criptografia de ponta a ponta: esta pessoa está em outro servidor.
e2ee_note_plain = Sem criptografia de ponta a ponta.
e2ee_plain_mark = Sem criptografia
e2ee_preview = Mensagem criptografada
e2ee_line_sealed = Mensagem criptografada
e2ee_line_own_key = Enviada para uma chave que você não tem mais
e2ee_line_peer_key = Enviada com uma chave em que você ainda não confiou
e2ee_line_unreadable = Não foi possível abrir
e2ee_line_unsupported = Este navegador não consegue abrir mensagens criptografadas
e2ee_block_peer_off = { $handle } ainda não ativou as mensagens criptografadas.
e2ee_block_self_off = Ative primeiro as mensagens criptografadas em Configurações.
e2ee_block_unlock = Desbloqueie primeiro as mensagens criptografadas em Configurações.
e2ee_block_downgrade = { $handle } já usou mensagens criptografadas. Esta conversa não voltará a mensagens sem criptografia.
e2ee_block_changed = Confie na nova chave de { $handle } para escrever para essa pessoa.
e2ee_block_error = Não foi possível verificar suas chaves de criptografia.
e2ee_changed_title = A chave de criptografia de { $name } mudou
e2ee_changed_body = A pessoa pode ter redefinido a criptografia ou migrado para um novo dispositivo, ou alguém pode estar no meio do caminho. Compare o novo número de segurança com ela antes de confiar. Você não pode enviar até fazer isso.
e2ee_changed_old = Número de segurança em que você confiava
e2ee_changed_new = Novo número de segurança
e2ee_trust = Confiar na nova chave
e2ee_safety_open = Número de segurança
e2ee_safety_title = Número de segurança
e2ee_safety_body = Compare este número com { $name } pessoalmente ou em uma ligação, não neste chat. Se for igual nas duas telas, ninguém está no meio.

## 0031
composer_upload_failed = Falha no envio. Tente de novo.
composer_send_failed = Não foi possível alcançar o tuita. Verifique sua conexão e tente de novo.
composer_open_failed = Não foi possível abrir o editor de publicações. Tente de novo.
