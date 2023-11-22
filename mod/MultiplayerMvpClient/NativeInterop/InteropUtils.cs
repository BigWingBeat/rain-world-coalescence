using System.Runtime.InteropServices;

namespace MultiplayerMvpClient.NativeInterop
{
	public unsafe static class InteropUtils
	{
		// Formats the given native error to a managed string and then drops the error.
		public static string FormatNativeError(Error* error)
		{
			ushort* errorPointer = Interop.format_error(error);
			string errorMessage = Marshal.PtrToStringUni(new(errorPointer));
			Interop.drop_string(errorPointer);
			Interop.drop_error(error);
			return errorMessage;
		}
	}
}
