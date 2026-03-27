# Reverse Engineering Guide

ThetaDataDx was built by decompiling ThetaData's Java terminal. This document covers how to repeat the process when ThetaData releases a new version.

## Source Terminal Version

| Field | Value |
|-------|-------|
| JAR version | `202603181` (2026-03-18, revision 1) |
| Size | 58.5 MB |
| Git commit | `85346bb` (branch: main) |
| Build author | William Speirs |
| Proto packages | `Endpoints` (shared types), `BetaEndpoints` (v3 MDDS service) |

## 1. Download the Latest Terminal

The terminal JARs are served by the Nexus API bootstrap endpoint. No authentication is required.

```bash
# List all available versions
curl -s https://nexus-api.thetadata.us/bootstrap/jars | python3 -m json.tool

# Grab the latest version string
VERSION=$(curl -s https://nexus-api.thetadata.us/bootstrap/jars \
    | python3 -c "import sys,json; print(json.load(sys.stdin)[-1])")
echo "Latest version: $VERSION"

# Download the JAR
curl -L -o terminal.jar "https://nexus-api.thetadata.us/bootstrap/jars/$VERSION"
```

The CDN endpoint is `https://td-terminals.nyc3.cdn.digitaloceanspaces.com/{version}.jar`.

## 2. Decompile with CFR

[CFR](https://www.benf.org/other/cfr/) is the recommended decompiler. You need JDK 21+.

```bash
# Download CFR
curl -L -o cfr-0.152.jar \
    "https://github.com/leibnitz27/cfr/releases/download/0.152/cfr-0.152.jar"

# Decompile only ThetaData packages
java -jar cfr-0.152.jar terminal.jar \
    --outputdir decompiled/ \
    --jarfilter "net.thetadata.*"
```

Key packages:

| Package | Contents |
|---------|----------|
| `net.thetadata.fpssclient/` | FPSS streaming protocol |
| `net.thetadata.fie/` | FIT/FIE codecs |
| `net.thetadata.auth/` | Nexus authentication |
| `net.thetadata.providers/` | MDDS gRPC channel setup |
| `net.thetadata.config/` | Configuration management |
| `net.thetadata.generated/` | Protobuf generated classes |

## 3. Extract Proto Definitions

The `.proto` files are not shipped as text. They are embedded as compiled `FileDescriptorProto` byte arrays. Use runtime reflection to extract them.

### DumpV3Proto.java

```java
import com.google.protobuf.DescriptorProtos;
import com.google.protobuf.Descriptors;

public class DumpV3Proto {
    public static void main(String[] args) throws Exception {
        Class<?> cls = Class.forName("net.thetadata.generated.v3grpc.Endpoints");
        java.lang.reflect.Method method = cls.getMethod("getDescriptor");
        Descriptors.FileDescriptor fd = (Descriptors.FileDescriptor) method.invoke(null);
        DescriptorProtos.FileDescriptorProto fdp = fd.toProto();
        System.out.println(fdp);
    }
}
```

### Running the extraction

```bash
# Extract classes from JAR
mkdir -p classes && cd classes && unzip ../terminal.jar > /dev/null && cd ..

# Compile the dumper
javac -cp terminal.jar DumpV3Proto.java -d dump/

# Extract v3 service definition
java -cp "dump/:classes/" DumpV3Proto > v3_endpoints.proto

# For shared types, modify to: Class.forName("net.thetadata.generated.Endpoints")
java -cp "dump/:classes/" DumpV3Proto > endpoints.proto
```

Place the extracted `.proto` files into `proto/` in the crate root. Run `cargo build` to regenerate Rust bindings.

## 4. Java Class to Rust Module Mapping

| Java Class | Contents | Rust Module |
|------------|----------|-------------|
| `FPSSClient` | FPSS lifecycle, auth, subscriptions, reconnection | `fpss/mod.rs` |
| `PacketStream` | Wire framing, request ID generation | `fpss/framing.rs` |
| `Contract` | Contract binary serialization | `fpss/protocol.rs` |
| `FITReader` | FIT nibble decoder | `codec/fit.rs` |
| `FIE` | FIE string encoder | `codec/fie.rs` |
| `UserAuthenticator` | Nexus HTTP auth, terminal key | `auth/nexus.rs` |
| `ChannelProvider` | MDDS gRPC channel (host, port, TLS) | `direct.rs` |
| `MddsConnectionManager` | MDDS v3 gRPC path | `config.rs` |
| `FpssConnectionManager` | FPSS multi-host failover | `fpss/connection.rs` |
| `StreamMsgType` | FPSS message type enum | `types/enums.rs` |
| `ConfigurationManager` | Config keys from `config_0.properties` | `config.rs` |

## 5. Hardcoded Constants

### Authentication

| Constant | Value |
|----------|-------|
| Nexus auth URL | `https://nexus-api.thetadata.us/identity/terminal/auth_user` |
| Terminal API key | `cf58ada4-4175-11f0-860f-1e2e95c79e64` |
| Terminal key header | `TD-TERMINAL-KEY` |

### MDDS

| Constant | Value |
|----------|-------|
| gRPC host | `mdds-01.thetadata.us` |
| gRPC port | `443` |
| gRPC service | `BetaEndpoints.BetaThetaTerminal` |
| RPC count | 60 methods (all server-streaming) |

### FPSS

| Constant | Value |
|----------|-------|
| NJ-A host:port | `nj-a.thetadata.us:20000`, `:20001` |
| NJ-B host:port | `nj-b.thetadata.us:20000`, `:20001` |
| Ping interval | 100ms |
| Reconnect delay (normal) | 2,000ms |
| Reconnect delay (rate limited) | 130,000ms |
| Connect timeout | 2,000ms |
| Read timeout | 10,000ms |
| TCP_NODELAY | `true` |

### FIT/FIE Codec

| Constant | Value |
|----------|-------|
| SPACING | 5 (ROW_SEP jumps to field index 5) |
| DATE marker | `0xCE` |
| FIELD_SEPARATOR | nibble `0xB` |
| ROW_SEPARATOR | nibble `0xC` |
| END | nibble `0xD` |
| NEGATIVE | nibble `0xE` |

## 6. Update Checklist

When ThetaData releases a new terminal version:

### Authentication

1. Check `UserAuthenticator.java` for changes to `CLOUD_AUTH_URL` or `TERMINAL_KEY`
2. Check auth request/response JSON format
3. Update `auth/nexus.rs` if changed

### gRPC / MDDS

1. Re-extract protos using steps 1-3 above
2. Diff proto files against previous versions
3. Add `define_endpoint!` invocations for new RPC methods
4. Update `decode.rs` parsers for changed types
5. Check `ChannelProvider.java` for host/port changes

### FPSS

1. Check `StreamMsgType.java` for new message codes
2. Check `FPSSClient.java` for state machine changes
3. Check `PacketStream.java` for framing changes
4. Check `Contract.java` for wire format changes
5. Check server address list for changes

### Codec

1. Check `FITReader.java` for encoding changes
2. Check `FIE.java` for nibble alphabet changes
3. Run test suite -- FIT test vectors catch encoding changes

### Configuration

1. Check `config_0.properties` for new defaults
2. Check `ConfigurationManager.java` for new keys

## 7. Useful Commands

```bash
# Search decompiled source
grep -r "session_uuid" decompiled/net/thetadata/

# Find all hardcoded URLs
grep -rn "https://" decompiled/net/thetadata/

# Find constants in auth code
grep -rn "static final" decompiled/net/thetadata/auth/

# Find FPSS message codes
grep -rn "StreamMsgType" decompiled/net/thetadata/

# List gRPC method names
grep -rn "getMethod" decompiled/net/thetadata/generated/v3grpc/

# Find config property keys
grep -rn "getProperty\|config_0" decompiled/net/thetadata/config/
```
