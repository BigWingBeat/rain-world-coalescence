using System.Runtime.InteropServices;
using MonoMod.Utils;
using MultiplayerMvpClient.Plugin;

namespace MultiplayerMvpClient.NativeInterop
{
	public static class MultiplayerMvpClientNative
	{
		public const string NATIVE_ASSEMBLY_NAME = "multiplayer_mvp_client";

#pragma warning disable CS8618 // The delegates are initialised by the `ResolveDynDllImports` method via reflection
		static MultiplayerMvpClientNative()
		{
			string pluginDirectory = Path.GetDirectoryName(MultiplayerMvpClientPlugin.PluginInstance.Info.Location);
			Dictionary<string, List<DynDllMapping>> mapping = new(1)
			{
				[NATIVE_ASSEMBLY_NAME] = new(1) {
					$"{pluginDirectory}\\..\\native\\{NATIVE_ASSEMBLY_NAME}.dll"
				}
			};

			typeof(MultiplayerMvpClientNative).ResolveDynDllImports(mapping);
		}
#pragma warning restore CS8618

		[UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
		public delegate void NativeErrorHandler(string error);

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void SetErrorHandler(NativeErrorHandler handler);

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void ResetToDefaultErrorHandler();

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void InitApp();

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		[return: MarshalAs(UnmanagedType.I1)]
		public delegate bool UpdateApp();

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void DestroyApp();

		[UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
		public delegate void ConnectToServer(string address, ushort port);

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void DestroyStaticTaskPools();

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate MovementDelta QueryMovementDelta();

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly SetErrorHandler set_error_handler;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly ResetToDefaultErrorHandler reset_to_default_error_handler;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly InitApp init_app;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly UpdateApp update_app;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly DestroyApp destroy_app;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly ConnectToServer connect_to_server;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly DestroyStaticTaskPools destroy_static_taskpools;

		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly QueryMovementDelta query_movement_delta;
	}

	[StructLayout(LayoutKind.Sequential)]
	public struct MovementDelta
	{
		public float x;
		public float y;

		public override string ToString()
		{
			return $"MovementDelta({x}, {y})";
		}
	}
}

