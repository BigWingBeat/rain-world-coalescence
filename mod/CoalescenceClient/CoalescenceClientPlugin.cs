global using CoalescenceClient.NativeInterop;
global using Plugin = CoalescenceClient.CoalescenceClientPlugin;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Runtime.Versioning;
using System.Security.Permissions;
using BepInEx;
using BepInEx.Logging;
using CoalescenceClient.Menu;
using UnityEngine;

#pragma warning disable CS0618 // SecurityAction.RequestMinimum is obsolete. However, this does not apply to the mod, which still needs it. Suppress the warning indicating that it is obsolete.
[assembly: SecurityPermission(SecurityAction.RequestMinimum, SkipVerification = true)]
#pragma warning restore CS0618

namespace CoalescenceClient
{
	[BepInPlugin(PLUGIN_GUID, PLUGIN_NAME, PLUGIN_VERSION)]
	public class CoalescenceClientPlugin : BaseUnityPlugin
	{
		public const string PLUGIN_GUID = "pixelstorm.coalescence.client";
		public const string PLUGIN_NAME = "Coalescence Client";
		public const string PLUGIN_VERSION = "0.1.0";

		/// <summary>
		/// The identifier used by the DllImports
		/// </summary>
		public const string NATIVE_ASSEMBLY_NAME = "coalescence_client";

		private static bool NativeLibraryLoaded = false;

#pragma warning disable CS8618 // The statics get initialised once in the instance constructor
		public static Plugin Instance { get; private set; }

		internal static new ManualLogSource Logger { get; private set; }
#pragma warning restore CS8618

		/// <summary>
		/// <see href="http://docs.go-mono.com/?link=xhtml%3adeploy%2fmono-api-unsorted.html">API docs</see>
		///
		/// <see href="https://github.com/Unity-Technologies/mono/blob/a0c23ad07814336ceaefc72efb858a0fa03610c6/mono/metadata/native-library.c#L246-L284">Source Code</see>
		/// </summary>
		[DllImport("__Internal", CharSet = CharSet.Ansi, ExactSpelling = true)]
		private static extern void mono_dllmap_insert(IntPtr assembly, string dll, string? func, string tdll, string? tfunc);

		private CoalescenceClientPlugin() : base()
		{
			Instance ??= this;
			Logger ??= base.Logger;
			LoadNativeLibrary();
		}

#pragma warning disable IDE0051, CA1822 // Unity uses reflection to call Awake, for this to work it must not be static
		private void Awake()
		{
			// PrintRuntimeInformation();

			unsafe { Interop.configure_native_logging(); }
			SetupHooks();
			ServerBrowserMenu.SetupHooks();
			ServerLobbyMenu.SetupHooks();
		}
#pragma warning restore IDE0051, CA1822

		public static string PluginDirectory()
		{
			return Path.GetDirectoryName(Instance.Info.Location);
		}

		public static string NativeAssemblyDirectory()
		{
			return Path.GetFullPath($"{PluginDirectory()}\\..\\native");
		}

		public static string NativeAssemblyPath()
		{
			return $"{NativeAssemblyDirectory()}\\{NATIVE_ASSEMBLY_NAME}.dll";
		}

		/// <summary>
		/// Creates a mapping from the identifier used in the DllImports to the actual location of the native library.
		///
		/// Must be called before any native methods can be called.
		/// Due to the way the JIT works, the call to this method cannot be in the same method body as any native method calls,
		/// or those native calls will fail anyway. <see href="https://stackoverflow.com/a/50256558">(Stack Overflow)</see>
		/// </summary>
		/// <remarks>
		/// This mapping is necessary because the path to the native DLL that would need to be specified in the DllImports can
		/// vary depending on where the mod is installed to, because it is interpreted relative to the Rain World executable,
		/// rather than relative to the mod's managed plugin DLL.
		/// </remarks>
		private static void LoadNativeLibrary()
		{
			if (!NativeLibraryLoaded)
			{
				string nativeAssemblyPath = NativeAssemblyPath();
				mono_dllmap_insert(IntPtr.Zero, NATIVE_ASSEMBLY_NAME, null, nativeAssemblyPath, null);
				NativeLibraryLoaded = true;
			}
		}

		private static void PrintRuntimeInformation()
		{
			Logger.LogInfo($"Environment.Version: '{Environment.Version}'");
			Logger.LogInfo($"RuntimeInformation.FrameworkDescription: '{RuntimeInformation.FrameworkDescription}'");

			var a1 = typeof(object).Assembly;
			var a2 = Assembly.GetEntryAssembly();
			var a3 = Assembly.GetExecutingAssembly();
			var a4 = typeof(RainWorld).Assembly;

			PrintAssemblyInfo(a1, "typeof(object).Assembly");
			PrintAssemblyInfo(a2, "Entry Assembly");
			PrintAssemblyInfo(a3, "Executing Assembly");
			PrintAssemblyInfo(a4, "typeof(RainWorld).Assembly");

			static void PrintAssemblyInfo(Assembly a, string displayName)
			{
				var name = a?.GetName()?.Name;
				var version = a?.GetName()?.Version;
				var aiv = a?.GetCustomAttribute<AssemblyInformationalVersionAttribute>();
				var tfm = a?.GetCustomAttribute<TargetFrameworkAttribute>();
				Logger.LogInfo($"Assembly '{displayName}' toString: '{a?.ToString() ?? "<null>"}'");
				Logger.LogInfo($"Assembly '{displayName}' simple name: '{name ?? "<null>"}'");
				Logger.LogInfo($"Assembly '{displayName}' version: '{version?.ToString() ?? "<null>"}'");
				Logger.LogInfo($"Assembly '{displayName}' AssemblyInformationalVersion: '{aiv?.InformationalVersion ?? "<null>"}'");
				Logger.LogInfo($"Assembly '{displayName}' TargetFramework.FrameworkDisplayName: '{tfm?.FrameworkDisplayName ?? "<null>"}'");
				Logger.LogInfo($"Assembly '{displayName}' TargetFramework.FrameworkName: '{tfm?.FrameworkName ?? "<null>"}'");
			}
		}

		private static void SetupHooks()
		{
			Application.quitting += DestroyStaticTaskPools;
		}

		/// <summary>
		/// Necessary to prevent the CLR softlocking when quitting
		/// </summary>
		private static void DestroyStaticTaskPools()
		{
			unsafe { Interop.terminate_taskpool_threads(); }
		}
	}
}

namespace System.Runtime.CompilerServices
{
	/// <summary>
	/// Required for init property accessors and primary constructors to compile
	/// <see href="https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.isexternalinit">(MSDN)</see>
	/// </summary>
	/// <seealso href="" />
	public class IsExternalInit;
}

namespace System.Runtime.InteropServices
{
	/// <summary>
	/// Used to annotate methods so they can be passed over FFI as function pointers, which has better performance than delegates
	/// <see href="https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.unmanagedcallersonlyattribute">(MSDN)</see> 
	/// </summary>
	[AttributeUsage(AttributeTargets.Method, Inherited = false)]
	public class UnmanagedCallersOnlyAttribute : Attribute
	{
		public Type[]? CallConvs;
		public string? EntryPoint;
	}
}
