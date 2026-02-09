// Only import specific properties to trigger tree-shaking
import { user } from './data.json';

// Only use user.name and user.profile.bio
// Other properties should be tree-shaken:
// - user.age, user.email
// - user.profile.avatar, user.profile.social
// - config (entire object)
// - metadata (entire object)
console.log('User name:', user.name);
console.log('User bio:', user.profile.bio);

it("should tree-shake unused JSON properties", () => {
  expect(user.name).toBe("Alice");
  expect(user.profile.bio).toBe("Software Engineer");

  // These should be tree-shaken and not exist
  expect(user.age).toBeUndefined();
  expect(user.email).toBeUndefined();
  expect(user.profile.avatar).toBeUndefined();
  expect(user.profile.social).toBeUndefined();
});
