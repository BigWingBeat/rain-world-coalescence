using System.Runtime.InteropServices;

namespace CoalescenceClient.NativeInterop
{
	public class SafeAppHandle : SafeHandle
	{
		public SafeAppHandle() : base(IntPtr.Zero, true) { }

		public unsafe SafeAppHandle(AppContainer* appHandle) : this()
		{
			SetHandle((IntPtr)appHandle);
		}

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

		/// <summary>
		/// Runs <see href="https://docs.rs/bevy/latest/bevy/app/struct.App.html#method.update">App::update</see> once
		/// </summary>
		/// <returns>Whether or not the app requested to exit this update</returns>
		public bool Update()
		{
			unsafe
			{
				return Convert.ToBoolean(Interop.update_app(AppHandle));
			}
		}

		/// <summary>
		/// Attempts to connect to a server
		/// </summary>
		/// <param name="address">The IP address or DNS name of the server</param>
		/// <param name="port">The port to connect to</param>
		/// <param name="username">This client's username</param>
		/// <param name="asyncOkHandler">Callback if the connection succeeded</param>
		/// <param name="asyncErrorHandler">Callback if the connection failed</param>
		/// <returns>Synchronous errors are returned directly, async errors invoke the <paramref name="asyncErrorHandler"/></returns>
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
