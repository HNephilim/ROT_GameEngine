# ROT_GameEngine
 ROT Game Engine
 
 Primeira tentativa (no github, deve ter sido a terceira ou quarta) de criar um GameEngine. 
 
Aprendi bastante sobre organização de código, renderização, APIs e até Multithread. Foi meu primeiro projeto mais complexo, fugiu do meu escopo.

O Entry point fic em ROT_App, contendo apenas alguns códigos de teste. A "magia" acontece em ROT_Engine. 

ROT_WGPU_Renderer é minha segunda tentativa com renderização em baixo nível e comunicação com GPU. Foi escrita de forma mais independente do que a de Vulkan (também disponível aqui no GitHub) utilizando o WGPU, uma biblioteca extremamente poderosa que consegue rodar nas principais bibliotecas gráficas de forma nativa com o mesmo código (Podemos compila-la para Vulkan, DirectX, Metal e até WebGL rodando em wasm (Web Assembly)
