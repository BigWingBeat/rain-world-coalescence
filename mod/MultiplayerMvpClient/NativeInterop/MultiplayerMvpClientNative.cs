using System.Runtime.InteropServices;
using MonoMod.Utils;

namespace MultiplayerMvpClient.NativeInterop
{
	public static class MultiplayerMvpClientNative
	{
		public const string NATIVE_ASSEMBLY_NAME = "multiplayer_mvp_client";

#pragma warning disable CS8618 // The delegates are initialised by the `ResolveDynDllImports` method via reflection
		static MultiplayerMvpClientNative()
		{
			string pluginDirectory = Path.GetDirectoryName(Plugin.Instance.Info.Location);
			Dictionary<string, List<DynDllMapping>> mapping = new(1)
			{
				[NATIVE_ASSEMBLY_NAME] = new(1) {
					$"{pluginDirectory}\\..\\native\\{NATIVE_ASSEMBLY_NAME}.dll"
				}
			};

			typeof(MultiplayerMvpClientNative).ResolveDynDllImports(mapping);
		}
#pragma warning restore CS8618

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate NewAppResult NewApp();
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly NewApp new_app;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		[return: MarshalAs(UnmanagedType.U1)]
		public delegate bool UpdateApp(IntPtr appHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly UpdateApp update_app;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
		public delegate IntPtr AppConnectToServer(IntPtr appHandle, string address, ushort port);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly AppConnectToServer app_connect_to_server;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void FreeApp(IntPtr appHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly FreeApp free_app;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl, CharSet = CharSet.Unicode)]
		public delegate string FormatError(IntPtr errorHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly FormatError format_error;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void FreeError(IntPtr errorHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly FreeError free_error;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate PollConnectionTaskResult PollConnectionTask(IntPtr taskHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly PollConnectionTask poll_connection_task;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void FreeConnectionTask(IntPtr taskHandle);
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly FreeConnectionTask free_connection_task;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate ushort DefaultPort();
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly DefaultPort default_port;

		[UnmanagedFunctionPointer(CallingConvention.Cdecl)]
		public delegate void TerminateTaskpoolThreads();
		[DynDllImport(NATIVE_ASSEMBLY_NAME)]
		public static readonly TerminateTaskpoolThreads terminate_taskpool_threads;
	}

	[StructLayout(LayoutKind.Explicit)]
	public struct NewAppResult
	{
		public enum NewAppResultTag : byte
		{
			App,
			Error
		}
		[FieldOffset(0)]
		public NewAppResultTag tag;
		[FieldOffset(4)]
		public IntPtr AppHandle;
		[FieldOffset(4)]
		public IntPtr ErrorHandle;
	}

	[StructLayout(LayoutKind.Explicit)]
	public struct PollConnectionTaskResult
	{
		public enum PollConnectionTaskResultTag : byte
		{
			IsCompleted,
			Error
		}
		[FieldOffset(0)]
		public PollConnectionTaskResultTag tag;
		[FieldOffset(1)]
		[MarshalAs(UnmanagedType.U1)]
		public bool IsCompleted;
		[FieldOffset(4)]
		public IntPtr ErrorHandle;
	}
}

