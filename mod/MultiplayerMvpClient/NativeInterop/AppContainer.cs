using System.Runtime.InteropServices;

namespace MultiplayerMvpClient.NativeInterop
{
	public sealed class AppContainer : SafeHandle
	{
		public AppContainer() : base(IntPtr.Zero, true) { }

		public override bool IsInvalid => IntPtr.Zero == handle;

		protected override bool ReleaseHandle()
		{
			throw new NotImplementedException();
		}
	}
}
