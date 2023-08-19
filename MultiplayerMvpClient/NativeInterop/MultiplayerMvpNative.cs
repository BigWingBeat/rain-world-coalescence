using System.Runtime.InteropServices;
using MonoMod.Utils;
using MultiplayerMvpClient.Plugin;

namespace MultiplayerMvpClient.NativeInterop
{
    public static class MultiplayerMvpNative
    {
        public const string NATIVE_ASSEMBLY_NAME = "multiplayer_mvp_native";

#pragma warning disable CS8618 // The delegates are initialised by the `ResolveDynDllImports` method via reflection
        static MultiplayerMvpNative()
        {
            Dictionary<string, List<DynDllMapping>> mapping = new(1)
            {
                [NATIVE_ASSEMBLY_NAME] = new(1) {
                    $"{Path.GetDirectoryName(MultiplayerMvpClientPlugin.Instance.Info.Location)}\\..\\native\\{NATIVE_ASSEMBLY_NAME}.dll"
                }
            };

            typeof(MultiplayerMvpNative).ResolveDynDllImports(mapping);
        }
#pragma warning restore CS8618

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void MovementCallback(float x_diff, float y_diff);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void InitApp(MovementCallback callback);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate bool UpdateApp();

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DestroyApp();

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void DestroyStaticTaskPools();

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static readonly InitApp init_app;

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static readonly UpdateApp update_app;

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static readonly DestroyApp destroy_app;

        [DynDllImport(NATIVE_ASSEMBLY_NAME)]
        public static readonly DestroyStaticTaskPools destroy_static_taskpools;
    }
}

