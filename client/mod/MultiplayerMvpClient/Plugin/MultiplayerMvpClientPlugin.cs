using BepInEx;
using BepInEx.Logging;
using MultiplayerMvpClient.NativeInterop;
using MultiplayerMvpClient.Plugin.Menu;
using UnityEngine;

namespace MultiplayerMvpClient.Plugin
{
	[BepInPlugin(MyPluginInfo.PLUGIN_GUID, MyPluginInfo.PLUGIN_NAME, MyPluginInfo.PLUGIN_VERSION)]
	public class MultiplayerMvpClientPlugin : BaseUnityPlugin
	{
		public const string PLUGIN_GUID = MyPluginInfo.PLUGIN_GUID;
		public const string PLUGIN_NAME = MyPluginInfo.PLUGIN_NAME;
		public const string PLUGIN_VERSION = MyPluginInfo.PLUGIN_VERSION;

#pragma warning disable CS8618 // Statics get populated in the constructor
		public static MultiplayerMvpClientPlugin Instance { get; private set; }

		internal static new ManualLogSource Logger { get; private set; }
#pragma warning restore CS8618

		private MultiplayerMvpClientPlugin() : base()
		{
			Instance ??= this;
			Logger ??= base.Logger;
		}

#pragma warning disable IDE0051, CA1822 // Unity uses reflection to call Awake, for this to work it must not be static
		private void Awake()
		{
			SetupHooks();
			MultiplayerLobby.SetupHooks();
		}
#pragma warning restore IDE0051, CA1822

		private static void SetupHooks()
		{
			Application.quitting += DestroyStaticTaskPools;
		}

		private static void DestroyStaticTaskPools()
		{
			MultiplayerMvpNative.destroy_static_taskpools();
		}
	}
}
