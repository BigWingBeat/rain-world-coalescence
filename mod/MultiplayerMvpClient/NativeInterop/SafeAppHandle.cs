using System.Runtime.InteropServices;

namespace MultiplayerMvpClient.NativeInterop
{
	public class SafeAppHandle : SafeHandle
	{
		public SafeAppHandle() : base(IntPtr.Zero, true) { }

		public unsafe SafeAppHandle(AppContainer* appHandle) : base((IntPtr)appHandle, true) { }

		private unsafe AppContainer* AppHandle => (AppContainer*)handle;

		public override bool IsInvalid => IntPtr.Zero == handle;

		protected override bool ReleaseHandle()
		{
			unsafe
			{
				Interop.drop_app(AppHandle);
			}

			return true;
		}

		// Returns whether or not the app requested to exit
		public bool Update()
		{
			unsafe
			{
				return Convert.ToBoolean(Interop.update_app(AppHandle));
			}
		}

		public unsafe AppConnectToServerResult ConnectToServer(string address, ushort port, string username, delegate* unmanaged[Cdecl]<void> asyncOkHandler, delegate* unmanaged[Cdecl]<Error*, void> asyncErrorHandler)
		{
			IntPtr addressPointer = Marshal.StringToHGlobalUni(address);
			IntPtr okCallbackPointer = (IntPtr)asyncOkHandler;
			IntPtr errorCallbackPointer = (IntPtr)asyncErrorHandler;
			IntPtr usernamePointer = Marshal.StringToHGlobalUni(username);
			AppConnectToServerResult result = Interop.app_connect_to_server(AppHandle, (ushort*)addressPointer, port, (ushort*)usernamePointer, okCallbackPointer, errorCallbackPointer);
			Marshal.FreeHGlobal(addressPointer);
			Marshal.FreeHGlobal(usernamePointer);
			return result;
		}
	}
}
