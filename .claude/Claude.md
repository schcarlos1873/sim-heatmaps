# Autoline Simulator

Paper profile scanners are online measurement systems like the Network Platform Scanners used in paper manufacturing to continuously track cross-direction (CD) and machine-direction (MD) properties across the moving web.

Core Scanner Models

• NP800 (Network Platform 800): Built for small-to-midsize paper and pulp machines with widths up to 8 meters.
• NP1200: Designed for wider, high-speed machines requiring high profile resolution and precise sensor alignment.
• RNP1200: Reflection network platform configuration for single-sided or specialized reflection-based measurement.

Key Profile Measurements

• Basis Weight: X-ray or nuclear sensors tracking mass per unit area.
• Moisture: High-performance infrared (HPIR) transmission/reflection or microwave sensors.
• Caliper: GT Caliper and optical sensors measuring thickness uniformity.
• Color & Optical: Spectrometer-based sensors monitoring color, brightness, opacity, and whiteness profiles.

Data Management & Control

• QCS Profile Historian: Collects, visualizes, and analyzes scanning data into 24-hour color-graded contour maps to spot variations.
• Automated CD/MD Control: Connects with profiling actuators (like steam, air-water, or induction profilers) to automatically correct sheet variations and reduce waste.
If you'd like, let me know:
• Which specific scanner model or measurement you are troubleshooting


## Objetivo

O objetivo é criar um simulador é gerar dados de scanner e uma pagina para visualizar o Mapa de Controle, com os gráficos para CD/MD.

Para gerar o Contour Map Profile (gráfico tridimensional ou mapa de calor do perfil do papel), o scanner da  envia uma matriz contínua de dados brutos e metadados estruturados.
Os principais dados enviados para o QCS (Quality Control System) são divididos em três categorias:

1. Dados de Medição dos Sensores (Variáveis de Processo)

O scanner envia leituras de alta frequência correspondentes a cada zona ou ponto da folha (data bins):
• Gramatura (Basis Weight): Variação de massa por unidade de área (g/m²).
• Umidade (Moisture): Percentual de água contido na folha (%).
• Espessura (Caliper): Medição micrométrica da espessura do papel (μ m).
• Cinzas/Componentes (Ash): Percentual de carga mineral (%).
• Propriedades Ópticas: Dados de cor (\(L^*, a^*, b^*\)), alvura e opacidade.

2. Dados de Posição Espacial e Sincronismo

Para que o software saiba exatamente onde a medição foi feita na folha:
• Posição do Carro (Cross-Direction - CD): A posição exata da cabeça do scanner no eixo transversal (em milímetros ou pelo número do data bin / zona do atuador).
• Velocidade da Máquina (Machine-Direction - MD): Velocidade da folha para calcular o avanço longitudinal enquanto o scanner se move.
• Borda da Folha (Sheet Edge Detection): Sinais que indicam exatamente onde começam e terminam a borda frontal (Front Side) e traseira (Back Side) do papel.
• Status do Turno/Scan: Identificação do número do corte (scan número), direção da varredura (ida ou volta) e tempo de início/fim do ciclo.

3. Dados de Diagnóstico e Qualidade do Sinal

Para garantir que o mapa represente dados reais e válidos:
• Status de Validação do Sensor: Código que confirma se o sensor estava calibrado, limpo e operando normalmente durante a passagem.
• Temperatura do Sensor: Dados térmicos internos para compensação de dilatação ou desvio de sinal.
• Indicador de Folha Rota (Sheet Break): Sinalizador que interrompe a alimentação do mapa caso o papel rasgue.

Como esses dados viram o Contour Map?

O sistema processa essas informações criando uma grade bidimensional (Matriz CD x MD):
1. O Eixo X representa a largura da máquina (dados de posição CD).
2. O Eixo Y representa o tempo ou comprimento produzido (dados de velocidade MD + tempo).
3. O Eixo Z (Cores) representa a intensidade ou desvio da medição (Gramatura, Umidade, etc.) em relação ao valor nominal (target).

## Documents

Utilize os documentos na pasta documents como referencia para desenvolver o simulador.

## Requisitos

- Cadastro das máquinas com as especifficações como largura pontos de leitura e quantidade onde o padrão deve ser 800.
- os pontos deve ser ajustado a largura da máquina.
- Cadastros das propriedades, com seus limites(Target, ErrorHight, ErrorLow, WarningHigh, WarningLow)  por máquina. (Basis Weight, Caliper,Moisture, L*, a*, b*, etc..)
- Gerador de dados por máquina.
- Visualizado do Contour Map dos dados dos arquivos gerados. 
- Exibir ao lado direto do contour o gráfico de linha e limites MD
- Exibir abaixo o CD e limites  
- Utilize a arquitetura neste diretório para manter o mesmo padrão de desenvolvimento.
- Substitua as referencias autoline por headmaps.  