# BedrockStation-rs

## ¿Como se ve la aplicación?

![](https://raw.githubusercontent.com/sergiotarxz/BedrockStation-rs/30347b0c531ca0f8a9523648501eaf66ed0a1cf9/mainscreen-screenshot.png)

## ¿Qué es esto?

Esta aplicación sirve para conectar Minecraft de PlayStation y posiblemente Xbox
a servidores de terceros usando un ordenador con Linux o Windows como proxy
para que Minecraft de Playstation crea que es otra máquina en la red local.

## ¿Como funciona?

Introduces la dirección del servidor y el puerto del mismo y la aplicación crea
un proxy UDP.

## Tecnologías usadas.

Rust, slint y más librerías de Cargo, lista completa en el archivo Cargo.toml.

## ¿Que licencia usa?

Mi código se puede usar sin restricciones bajo los terminos de AGPLv3, aunque
se ha realizado esfuerzo en que todas las dependencias sean compatibles con
la licencia no puedo ofrecer ninguna garantia al respecto, por lo que deberías
consultar con un abogado en caso de tener algún tipo de temor legal.

## Como lo ejecuto:

Si usas Windows lo recomendable es directamente descargar el .exe de esta
url (https://github.com/sergiotarxz/BedrockStation-rs/releases) si usas
Linux directamente puedes compilar la aplicación y ejecutarla usando los
siguientes comandos:

```shell
git clone https://github.com/sergiotarxz/BedrockStation-rs
cd BedrockStation-rs
cargo run
```

Si hubiese interés en crear una versión para flathub se podría hacer con
relativamente poco esfuerzo, crea una issue en Github si te interesa algo
así.

## ¿Es este software compatible con servidores de Minecraft detrás de una VPN?

Sí, no obstante el soporte de conectarse a la VPN no viene incluido en el software,
deberás descargar por separado software como Wireguard, Openvpn o lo que quiera
que tu servidor de Minecraft necesite.

## ¿Es este software compatible con proxies como Geyser?

Sí, este software permite conectarte a través de proxies Geyser a servidores
de Minecraft Java; no obstante es muy probable que el propio administrador del servidor
tenga que ser quien configure ese software.

## Recuerda que este software no tiene ningún tipo de garantía.

He testeado suficientemente este software para pensar que es estable y
seguro de ejecutar; no obstante no te puedo ofrecer ningún tipo de
garantía legal al respecto.

Si el software da algún tipo de problema puedes abrir un issue o incidencia;
no obstante no te puedo asegurar poderte ayudar.

## Descargo de responsabilidad.

Minecraft™ es una marca registrada por Microsoft y Mojang; y este software ni
sus desarrolladores están afiliado con esas empresas de ningún modo.


## Why all the README.md and App is in Spanish?

Because this app was mostly made for my friends that would get scared
otherwise and this is their main way to download the app.

