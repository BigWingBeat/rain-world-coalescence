using System.Runtime.InteropServices;

namespace MultiplayerMvpClient.NativeInterop
{
	public unsafe static class InteropUtils
	{
		/// <summary>
		/// Formats the given native error to a managed string and then drops the error.
		/// </summary>
		/// <param name="error">The native error to format. Becomes a dangling pointer after the method returns</param>
		/// <returns>The error message associated with the given error</returns>
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
