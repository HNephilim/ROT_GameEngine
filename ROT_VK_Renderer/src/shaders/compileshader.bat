
@echo off

C:\\VulkanSDK\\1.2.170.0\\Bin\\glslc vertex.vert -o vert.spv
C:\\VulkanSDK\\1.2.170.0\\Bin\\glslc fragment.frag -o frag.spv

MOVE frag.spv D:\ROT_GameEngine\shaders
MOVE vert.spv D:\ROT_GameEngine\shaders