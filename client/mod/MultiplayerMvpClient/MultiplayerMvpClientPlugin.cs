global using Plugin = MultiplayerMvpClient.MultiplayerMvpClientPlugin;
global using Interop = MultiplayerMvpClient.NativeInterop.MultiplayerMvpClientNative;

using BepInEx;
using BepInEx.Logging;
using MultiplayerMvpClient.Menu;
using UnityEngine;

namespace MultiplayerMvpClient
{
	[BepInPlugin(PLUGIN_GUID, PLUGIN_NAME, PLUGIN_VERSION)]
	public class MultiplayerMvpClientPlugin : BaseUnityPlugin
	{
		public const string PLUGIN_GUID = "pixelstorm.multiplayer_mvp";
		public const string PLUGIN_NAME = "Multiplayer MVP Client";
		public const string PLUGIN_VERSION = "0.1.0";

#pragma warning disable CS8618 // Statics get populated in the constructor
		public static Plugin Instance { get; private set; }

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
			Interop.destroy_static_taskpools();
		}
	}
}
