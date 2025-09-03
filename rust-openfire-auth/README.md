# OpenFire Authentication Library in Rust

Este projeto implementa autenticação e comunicação com servidores OpenFire usando Rust, conforme solicitado. A biblioteca fornece uma interface JNI para integração com o código Java existente do Spark.

## Estrutura do Projeto

```
rust-openfire-auth/
├── src/
│   ├── lib.rs              # Módulo principal da biblioteca
│   ├── auth.rs             # Gerenciamento de autenticação
│   ├── communication.rs    # Comunicação XMPP e OpenFire
│   ├── config.rs           # Gerenciamento de configuração
│   ├── error.rs            # Tipos de erro customizados
│   └── jni_interface.rs    # Interface JNI para Java
├── Cargo.toml              # Configuração do projeto Rust
└── README.md               # Esta documentação

java-interface/
└── src/main/java/org/jivesoftware/spark/openfire/
    ├── OpenFireAuthNative.java    # Interface JNI nativa
    ├── OpenFireAuthClient.java    # Cliente Java de alto nível
    └── OpenFireAuthExample.java   # Exemplo de uso
```

## Funcionalidades Implementadas

### Módulo de Autenticação (`auth.rs`)
- ✅ Gerenciamento de credenciais de usuário
- ✅ Validação de credenciais
- ✅ Estados de autenticação (Conectado, Autenticando, Autenticado, Falhou)
- ✅ Resultado de autenticação com informações detalhadas
- ✅ Suporte a timeout de autenticação

### Módulo de Comunicação (`communication.rs`)
- ✅ Cliente XMPP para OpenFire
- ✅ Envio de mensagens de chat
- ✅ Gerenciamento de presença (Disponível, Ausente, Não Perturbe, etc.)
- ✅ Suporte a salas de chat (Multi-User Chat)
- ✅ Gerenciamento de contatos (roster)
- ✅ Sistema de eventos para callbacks
- ✅ Informações do servidor

### Módulo de Configuração (`config.rs`)
- ✅ Configuração flexível via TOML ou JSON
- ✅ Validação de configuração
- ✅ Suporte a TLS/SSL
- ✅ Configuração de timeouts
- ✅ Prioridade de presença

### Interface JNI (`jni_interface.rs`)
- ✅ Inicialização da biblioteca
- ✅ Criação e destruição de clientes
- ✅ Conexão e autenticação
- ✅ Envio de mensagens
- ✅ Gerenciamento de presença
- ✅ Entrada em salas de chat
- ✅ Conversão segura entre Java e Rust

### Tratamento de Erros (`error.rs`)
- ✅ Tipos de erro específicos para OpenFire
- ✅ Mapeamento de erros de bibliotecas externas
- ✅ Mensagens de erro informativas

## Dependências Principais

- **tokio**: Runtime assíncrono para operações de rede
- **xmpp/tokio-xmpp**: Biblioteca XMPP para comunicação
- **serde**: Serialização/deserialização de dados
- **jni**: Interface com Java
- **rustls**: Suporte TLS/SSL
- **anyhow/thiserror**: Tratamento de erros
- **log**: Sistema de logging

## Como Compilar

### Pré-requisitos
- Rust 1.70+ instalado
- Java 8+ instalado
- Maven (para compilar interface Java)

### Compilar a biblioteca Rust
```bash
cd rust-openfire-auth
cargo build --release
```

### Executar testes
```bash
cd rust-openfire-auth
cargo test
```

### Compilar interface Java (opcional)
```bash
cd java-interface
javac -cp ".:gson.jar" src/main/java/org/jivesoftware/spark/openfire/*.java
```

## Como Usar

### Exemplo básico em Java

```java
// Inicializar biblioteca
OpenFireAuthClient.initialize();

// Configuração
OpenFireAuthClient.Config config = new OpenFireAuthClient.Config("servidor.openfire.com", "dominio.com");
config.port = 5222;
config.use_tls = true;

// Criar cliente
OpenFireAuthClient client = new OpenFireAuthClient(config);

// Conectar e autenticar
OpenFireAuthClient.AuthResult result = client.connect("usuario", "senha", "dominio.com");

if (result.success) {
    // Definir presença
    client.setPresence(OpenFireAuthClient.PRESENCE_AVAILABLE, "Conectado via Rust!");
    
    // Enviar mensagem
    String messageId = client.sendMessage("contato@dominio.com", "Olá do Rust!");
    
    // Entrar em sala
    client.joinRoom("sala@conference.dominio.com", "MeuNick");
}

// Limpar recursos
client.close();
```

### Configuração via JSON

```json
{
    "server": "openfire.servidor.com",
    "port": 5222,
    "domain": "dominio.com",
    "use_tls": true,
    "verify_certificates": true,
    "connection_timeout": 30,
    "auth_timeout": 10,
    "resource": "SparkRust",
    "priority": 1
}
```

## Integração com Spark Existente

Para integrar esta biblioteca com o código Spark existente:

1. **Copie os arquivos Rust** para a pasta `rust-openfire-auth/`
2. **Compile a biblioteca** usando `cargo build --release`
3. **Adicione as classes Java** ao projeto Spark
4. **Configure o classpath** para incluir a biblioteca nativa
5. **Substitua as chamadas de autenticação** existentes pelos novos métodos

### Exemplo de integração no SessionManager:

```java
// Em SessionManager.java
import org.jivesoftware.spark.openfire.OpenFireAuthClient;

public class SessionManager {
    private OpenFireAuthClient rustClient;
    
    public void authenticateWithRust(String username, String password, String server) {
        OpenFireAuthClient.Config config = new OpenFireAuthClient.Config(server, server);
        rustClient = new OpenFireAuthClient(config);
        
        OpenFireAuthClient.AuthResult result = rustClient.connect(username, password, server);
        if (result.success) {
            // Autenticação bem-sucedida
            setConnection(createXMPPConnection(result));
        }
    }
}
```

## Vantagens desta Implementação

1. **Performance**: Rust oferece performance próxima ao C/C++
2. **Segurança**: Sistema de tipos do Rust previne bugs de memória
3. **Concorrência**: Modelo assíncrono do Rust é ideal para redes
4. **Interoperabilidade**: Interface JNI permite uso transparente do Java
5. **Manutenibilidade**: Código Rust é mais fácil de manter que JNI em C
6. **Modularidade**: Separação clara entre autenticação e comunicação

## Estrutura de Diretórios no Projeto Spark

```
Spark_junonns/
├── core/                    # Código Java existente
├── plugins/                 # Plugins existentes
├── rust-openfire-auth/      # ✅ Nova biblioteca Rust
│   ├── src/
│   ├── Cargo.toml
│   └── README.md
├── java-interface/          # ✅ Interface Java para Rust
│   └── src/main/java/
└── pom.xml                  # Configuração Maven existente
```

## Próximos Passos

Para finalizar a integração:

1. Compilar a biblioteca para diferentes plataformas (Windows, Linux, macOS)
2. Adicionar testes de integração com servidor OpenFire real
3. Implementar callbacks para eventos em tempo real
4. Otimizar performance e uso de memória
5. Adicionar suporte a plugins específicos do OpenFire

## Suporte e Manutenção

Esta implementação segue as melhores práticas de Rust e fornece uma base sólida para autenticação e comunicação com OpenFire. O código é modular, testável e preparado para extensões futuras.