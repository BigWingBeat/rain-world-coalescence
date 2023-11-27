using UnityEngine;

namespace CoalescenceClient.Menu
{
	public readonly struct ScreenDimensions
	{
		public readonly Vector2 ScreenSize;
		public readonly Vector2 ScreenCenter;

		public readonly Vector2 BottomLeft => Vector2.zero;
		public readonly Vector2 TopRight => ScreenSize;

		public ScreenDimensions(ProcessManager manager)
		{
			ScreenSize = manager.rainWorld.screenSize;
			ScreenCenter = ScreenSize / 2;
		}
	}
}
