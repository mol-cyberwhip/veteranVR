package dev.veteran.quest.app.platform

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.content.pm.PackageInstaller
import android.os.Build
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.concurrent.ConcurrentHashMap

class PackageInstallerBridge(
    private val context: Context,
) {
    companion object {
        const val ACTION_INSTALL_RESULT = "dev.veteran.quest.app.ACTION_INSTALL_RESULT"
        const val ACTION_UNINSTALL_RESULT = "dev.veteran.quest.app.ACTION_UNINSTALL_RESULT"
        const val EXTRA_OPERATION_ID = "operation_id"
    }

    private val waiters = ConcurrentHashMap<String, CompletableDeferred<Result<String>>>()
    private val lock = Mutex()
    private var registered = false

    private val receiver = object : BroadcastReceiver() {
        override fun onReceive(ctx: Context, intent: Intent) {
            val opId = intent.getStringExtra(EXTRA_OPERATION_ID) ?: return
            val waiter = waiters[opId] ?: return

            val status = intent.getIntExtra(PackageInstaller.EXTRA_STATUS, PackageInstaller.STATUS_FAILURE)
            val message = intent.getStringExtra(PackageInstaller.EXTRA_STATUS_MESSAGE).orEmpty()

            when (status) {
                PackageInstaller.STATUS_PENDING_USER_ACTION -> {
                    val confirmationIntent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        intent.getParcelableExtra(Intent.EXTRA_INTENT, Intent::class.java)
                    } else {
                        @Suppress("DEPRECATION")
                        intent.getParcelableExtra(Intent.EXTRA_INTENT)
                    }

                    if (confirmationIntent != null) {
                        confirmationIntent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        context.startActivity(confirmationIntent)
                    }
                }

                PackageInstaller.STATUS_SUCCESS -> {
                    waiter.complete(Result.success(message.ifBlank { "Success" }))
                    waiters.remove(opId)
                }

                else -> {
                    waiter.complete(Result.failure(IllegalStateException(message.ifBlank { "Package installer failed" })))
                    waiters.remove(opId)
                }
            }
        }
    }

    suspend fun ensureRegistered() {
        lock.withLock {
            if (registered) {
                return
            }
            val filter = IntentFilter().apply {
                addAction(ACTION_INSTALL_RESULT)
                addAction(ACTION_UNINSTALL_RESULT)
            }
            context.registerReceiver(receiver, filter)
            registered = true
        }
    }

    suspend fun newWaiter(operationId: String): CompletableDeferred<Result<String>> {
        ensureRegistered()
        return CompletableDeferred<Result<String>>().also { deferred ->
            waiters[operationId] = deferred
        }
    }

    fun installIntentSender(operationId: String): android.content.IntentSender {
        val intent = Intent(ACTION_INSTALL_RESULT).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_OPERATION_ID, operationId)
        }
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }

        val pendingIntent = PendingIntent.getBroadcast(context, operationId.hashCode(), intent, flags)
        return pendingIntent.intentSender
    }

    fun uninstallIntentSender(operationId: String): android.content.IntentSender {
        val intent = Intent(ACTION_UNINSTALL_RESULT).apply {
            setPackage(context.packageName)
            putExtra(EXTRA_OPERATION_ID, operationId)
        }
        val flags = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
        } else {
            PendingIntent.FLAG_UPDATE_CURRENT
        }

        val pendingIntent = PendingIntent.getBroadcast(context, operationId.hashCode(), intent, flags)
        return pendingIntent.intentSender
    }
}
