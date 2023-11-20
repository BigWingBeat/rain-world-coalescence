global using MultiplayerMvpClient.NativeInterop;
global using Plugin = MultiplayerMvpClient.MultiplayerMvpClientPlugin;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Runtime.Versioning;
using System.Security.Permissions;
using BepInEx;
using BepInEx.Logging;
using MultiplayerMvpClient.Menu;
using UnityEngine;

#pragma warning disable CS0618 // SecurityAction.RequestMinimum is obsolete. However, this does not apply to the mod, which still needs it. Suppress the warning indicating that it is obsolete.
[assembly: SecurityPermission(SecurityAction.RequestMinimum, SkipVerification = true)]
#pragma warning restore CS0618

namespace MultiplayerMvpClient
{
	[BepInPlugin(PLUGIN_GUID, PLUGIN_NAME, PLUGIN_VERSION)]
	public class MultiplayerMvpClientPlugin : BaseUnityPlugin
	{
		public const string PLUGIN_GUID = "pixelstorm.coalescence.client";
		public const string PLUGIN_NAME = "Multiplayer MVP Client";
		public const string PLUGIN_VERSION = "0.1.0";

		public const string NATIVE_ASSEMBLY_NAME = "multiplayer_mvp_client";

		private static bool NativeLibraryLoaded = false;

#pragma warning disable CS8618 // The statics get initialised once in the instance constructor
		public static Plugin Instance { get; private set; }

		internal static new ManualLogSource Logger { get; private set; }
#pragma warning restore CS8618

		// http://docs.go-mono.com/?link=xhtml%3adeploy%2fmono-api-unsorted.html
		[DllImport("__Internal", CharSet = CharSet.Ansi, ExactSpelling = true)]
		private static extern void mono_dllmap_insert(IntPtr assembly, string dll, string? func, string tdll, string? tfunc);

		private MultiplayerMvpClientPlugin() : base()
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

		// https://stackoverflow.com/a/50256558
		private static void LoadNativeLibrary()
		{
			if (!NativeLibraryLoaded)
			{
				string nativeAssemblyPath = NativeAssemblyPath();
				mono_dllmap_insert(IntPtr.Zero, NATIVE_ASSEMBLY_NAME, null, nativeAssemblyPath, null);
				NativeLibraryLoaded = true;
			}
		}

		public static void PrintRuntimeInformation()
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

		private static void DestroyStaticTaskPools()
		{
			unsafe { Interop.terminate_taskpool_threads(); }
		}
	}
}
