/* Select the published launch/sched C ABI family for this translation unit.
   Existing consumers default to 0.8. Define BENNETT_ABI_FAMILY as 9 before
   including this file to select 0.9. See README.md for provider selection. */
#ifndef BENNETT_ABI_FAMILY
#define BENNETT_ABI_FAMILY 8
#endif

/* Check even on repeated includes: changing families must fail loudly. */
#if BENNETT_ABI_FAMILY == 8
#if defined(BENNETT_LAUNCH_0_9_H) || defined(BENNETT_SCHED_0_9_H)
#error "Bennett C ABI families 0.8 and 0.9 cannot be mixed in one translation unit"
#endif
#elif BENNETT_ABI_FAMILY == 9
#if defined(BENNETT_LAUNCH_H) || defined(BENNETT_SCHED_H)
#error "Bennett C ABI families 0.8 and 0.9 cannot be mixed in one translation unit"
#endif
#else
#error "BENNETT_ABI_FAMILY must be 8 (ABI 0.8) or 9 (ABI 0.9)"
#endif

#ifndef BENNETT_ABI_H
#define BENNETT_ABI_H

#if BENNETT_ABI_FAMILY == 8
#include "bennett_launch.h"
#include "bennett_sched.h"
#elif BENNETT_ABI_FAMILY == 9
#include "bennett_launch_0_9.h"
#include "bennett_sched_0_9.h"
#endif

#endif /* BENNETT_ABI_H */
